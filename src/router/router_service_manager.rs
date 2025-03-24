use std::path::{self, PathBuf};
use std::{future::Future, path::Path};
use std::sync::Arc;
use actix::{spawn, Actor, Addr};
use tracing::{info, trace};
use notify::{Config, Error, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;

use crate::{mcp::{ListPromptsActor, ListToolsActor, ListResourcesActor}, messages::{AddPromptsRequest, AddResourcesRequest, AddToolsRequest}};
use super::{wasm_router::spawn_wasm_router, WasmRouter};
use super::{router_registry::{ActorRouterRegistry, RouterRegistry}, Router, RouterActor, SystemRouter};

pub enum RegistryType {
    Native,
    Wasi,
}
#[derive(Clone)]
pub struct RouterServiceManager {
    list_prompts: Addr<ListPromptsActor>,
    list_tools: Addr<ListToolsActor>,
    list_resources: Addr<ListResourcesActor>,
    active_registry: ActorRouterRegistry,
    wasm_path: Option<String>, // Optional directory to monitor for Wasm files
}

impl RouterServiceManager {
    fn new(wasm_path: Option<String>) -> Self {
        let active_registry = ActorRouterRegistry::new();
        let list_prompts = ListPromptsActor::new().start();
        let list_tools = ListToolsActor::new().start();
        let list_resources = ListResourcesActor::new().start();

        Self {
            list_prompts,
            list_tools,
            list_resources,
            active_registry,
            wasm_path,
        }
    }

    pub async fn default(wasm_path: Option<String>) -> Self {

        let mut manager = RouterServiceManager::new(wasm_path.clone());
        let system = SystemRouter::new();
        let _ = manager
            .register_router::<SystemRouter>("system".to_string(), Box::new(system))
            .await;
        
        // Optionally handle the wasm directory at startup by registering all existing wasm routers
        if let Some(path) = wasm_path {
            let wpath = Arc::new(path);
            manager.clone().scan_and_register_wasm_files(wpath.clone()).await;
            manager.clone().watch_wasm_directory(wpath.clone());
        }

        manager
    }

    // Watch the wasm directory for changes (create, update, rename, remove)
    fn watch_wasm_directory(&mut self, wasm_path: Arc<String>) -> Result<(), Box<dyn std::error::Error>>
    {
        let (_tx, rx) = channel::<Event>();

        let mut watcher = RecommendedWatcher::new(move |result: Result<Event, Error>| {
            match result {
                Ok(event) => {
                    match event.kind {
                        // We only care about the creation, modification, and removal of .wasm files
                        notify::EventKind::Modify(_) | notify::EventKind::Create(_) | notify::EventKind::Remove(_) => {
                            if let Some(path) = event.paths.first() {
                                if path.extension() == Some("wasm".as_ref()) {
                                    let file = path.as_path().to_str().unwrap();
                                    let router_id = path.file_stem().unwrap().to_string_lossy().into_owned();
                                    // Call the respective function based on event type
                                    let future = match event.kind {
                                        notify::EventKind::Create(notify::event::CreateKind::File) => {
                                            info!("Wasm file created: {:?}", path);
                                            let router = create_wasm_router(path);
                                            self.register_router::<WasmRouter>(router_id.clone(), router)
                                        }
                                        notify::EventKind::Modify(notify::event::ModifyKind::Any) => {
                                            info!("Wasm file modified: {:?}", path);
                                            self.unregister_router(&router_id);
                                            let new_router = create_wasm_router(path);
                                            self.register_router::<WasmRouter>(router_id.clone(), new_router)
                                        }
                                        notify::EventKind::Remove(notify::event::RemoveKind::File) => {
                                            info!("Wasm file removed: {:?}", path);
                                            self.unregister_router(&router_id).await;
                                            return ()
                                        }
                                        _ => return (), // Ignore other types of events
                                    };
        
                                    // If a future was returned, we use it (for async operations)
                                    Box::pin(future)
                                } else {
                                    // not a wasm file, ignore
                                    return ()
                                }
                            } else {
                                // not a valid file, ignore
                                return ()
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error watching directory: {:?}", e);
                }
            }
        }, Config::default())?;

        watcher.watch(Path::new(wasm_path.as_ref()), RecursiveMode::Recursive)?;

        // Watch the directory in a background task
        actix::spawn(async move {
            loop {
                match rx.recv() {
                    Ok(_) => {} // The logic is handled inside the watcher callback
                    Err(e) => eprintln!("Error: {:?}", e),
                }
            }
        });

        Ok(())
    }

    // Recursively find all Wasm files in the directory and register them
    async fn scan_and_register_wasm_files(&mut self, wasm_path: Arc<String>) {
        let paths = std::fs::read_dir(Path::new(wasm_path.as_ref())).unwrap();

        for entry in paths {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                    let router = create_wasm_router(&path);
                    let router_id = path.to_str().unwrap().to_string();
                    self.register_router::<WasmRouter>(router_id, router).await;
                }
            }
        }
    }

    // Register the router
    pub async fn register_router<T: Router>(&mut self, router_id: String, router: Box<dyn Router>) -> Result<(), String> {
        let tools = router.list_tools();
        let resources = router.list_resources();
        let prompts = router.list_prompts();
        let _capabilities = &router.capabilities().clone();
        let router_addr = RouterActor::new(Arc::new(router)).start();

        info!("Registering router {} at {:?}", router_id.clone(), router_addr.clone());
        self.active_registry.register_router(router_id.clone(), router_addr.clone())?;

        self.list_prompts.do_send(AddPromptsRequest {
            router_id: router_id.clone(),
            prompts,
            router: router_addr.clone(),
        });
        self.list_tools.do_send(AddToolsRequest {
            router_id: router_id.clone(),
            tools,
            router: router_addr.clone(),
        });
        self.list_resources.do_send(AddResourcesRequest {
            router_id: router_id.clone(),
            resources,
            router: router_addr.clone(),
        });

        Ok(())
    }

    // Unregister the router
    pub async fn unregister_router(&mut self, router_id: &str) -> Result<(), String> {
        // Unregister the router
        self.active_registry.unregister_router(router_id);

        info!("Unregistered router: {}", router_id);
        Ok(())
    }

    pub async fn get_router(&self, action: String) -> (Option<Addr<RouterActor>>, String) {
        self.active_registry.get_router(action.clone())
    }

    pub fn get_registry(&self) -> ActorRouterRegistry {
        self.active_registry.clone()
    }

    pub fn get_list_prompts(&self) -> Addr<ListPromptsActor> {
        self.list_prompts.clone()
    }

    pub fn get_list_resources(&self) -> Addr<ListResourcesActor> {
        self.list_resources.clone()
    }

    pub fn get_list_tools(&self) -> Addr<ListToolsActor> {
        self.list_tools.clone()
    }
}

// Helper function to create a Wasm router
fn create_wasm_router(path: &std::path::Path) -> Box<WasmRouter> {
    // Here, you should implement the logic to create the router
    // This is a simplified version
    let handle = spawn_wasm_router(path.to_str().unwrap());
    let router = WasmRouter::new(handle);
    //Box::new(WasmRouter::new(path.to_str().unwrap()).expect(format!("could not create wasm router for {:?}",path.clone()).as_str()))
    Box::new(router)
}
