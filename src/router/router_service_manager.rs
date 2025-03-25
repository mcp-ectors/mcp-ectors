use std::path::Path;
use std::sync::{Arc, Mutex};
use actix::{Actor, Addr};
use tracing::info;
use notify::{Error, Event, EventKind, RecommendedWatcher, Watcher};
use crate::messages::{GetRouter, RegisterRouter, UnregisterRouter};
use crate::{mcp::{ListPromptsActor, ListToolsActor, ListResourcesActor}, messages::{AddPromptsRequest, AddResourcesRequest, AddToolsRequest}};
use super::wasm_router::spawn_wasm_router;
use super::WasmRouter;
use super::{router_registry::ActorRouterRegistry, Router, RouterActor, SystemRouter};

pub enum RegistryType {
    Native,
    Wasi,
}
#[derive(Clone)]
pub struct RouterServiceManager {
    list_prompts: Addr<ListPromptsActor>,
    list_tools: Addr<ListToolsActor>,
    list_resources: Addr<ListResourcesActor>,
    active_registry: Addr<ActorRouterRegistry>,
}

impl RouterServiceManager {
    fn new() -> Self {
        let active_registry = ActorRouterRegistry::new().start();
        let list_prompts = ListPromptsActor::new().start();
        let list_tools = ListToolsActor::new().start();
        let list_resources = ListResourcesActor::new().start();

        Self {
            list_prompts,
            list_tools,
            list_resources,
            active_registry,
        }
    }

    pub async fn default(wasm_path: Option<String>) -> Self {

        let mut manager = RouterServiceManager::new();
        let system = SystemRouter::new();
        let _ = manager
            .register_router::<SystemRouter>("system".to_string(), Box::new(system))
            .await;
        
        // Optionally handle the wasm directory at startup by registering all existing wasm routers
        if let Some(path) = wasm_path {
            let wpath = Arc::new(path);
            manager.clone().scan_and_register_wasm_files(wpath.clone()).await;
            // Spawn the directory watch on a separate async task
            let _wpath_clone = wpath.clone();
            let _watcher = manager.clone();
            /*tokio::task::spawn(async move {
                if let Err(e) = watcher.watch_wasm_directory(wpath_clone).await {
                    error!("Error watching directory: {:?}", e);
                }
            });
            */
        }

        manager
    }

    // Watch the wasm directory for changes (create, update, rename, remove)
    async fn _watch_wasm_directory(&self, wasm_path: Arc<String>) -> Result<(), Box<dyn std::error::Error>> {
        let (_tx, rx) = std::sync::mpsc::channel::<Event>();
        let rsm = Arc::new(Mutex::new(self.clone()));
        let mut watcher = RecommendedWatcher::new(move |result: Result<Event, Error>| {
            match result {
                Ok(event) => {
                    match event.kind {
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_) => {
                            if let Some(path) = event.paths.first() {
                                if let Some("wasm") = path.extension().and_then(|ext| ext.to_str()) {
                                    //let router_id = path.file_stem().unwrap().to_string_lossy().into_owned();
                                    let router_id = path.file_stem()
                                        .and_then(|name| name.to_str())  // Get the file name without the extension
                                        .unwrap_or("defaultname")       // Provide a default name in case of failure
                                        .replace('_', "")               // Replace all underscores
                                        .to_string();
                                    match event.kind {
                                        EventKind::Create(_) => {
                                            println!("Wasm file created: {:?}", path);
                                            let router = create_wasm_router(path);
                                            let _ = rsm.lock().unwrap().register_router::<WasmRouter>(router_id.clone(),router);
                                        }
                                        EventKind::Modify(_) => {
                                            // this gets called twice. One time with the old and one time with the new
                                            if path.exists() {
                                                println!("Wasm file modified - new name: {:?}", path);
                                                let router = create_wasm_router(path);
                                                let _ = rsm.lock().unwrap().register_router::<WasmRouter>(router_id.clone(),router);
                                            } else {
                                                println!("Wasm file modified - oldname: {:?}", path);
                                                let _ = rsm.lock().unwrap().unregister_router(&router_id);
                                            }
                                        }
                                        EventKind::Remove(_) => {
                                            println!("Wasm file removed: {:?}", path);
                                            let _ = rsm.lock().unwrap().unregister_router(&router_id);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Err(e) => {
                    eprintln!("Error watching directory: {:?}", e);
                }
            }
        }, notify::Config::default())?;

        watcher.watch(Path::new(wasm_path.as_ref()), notify::RecursiveMode::Recursive)?;

        // Loop to keep the watcher alive
        loop {
            rx.recv().unwrap();
        }
    }


    // Recursively find all Wasm files in the directory and register them
    async fn scan_and_register_wasm_files(&mut self, wasm_path: Arc<String>) {
        let paths = std::fs::read_dir(Path::new(wasm_path.as_ref())).unwrap();

        for entry in paths {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
                    let router = create_wasm_router(&path);
                    let router_id = path.file_stem()
                        .and_then(|name| name.to_str())  // Get the file name without the extension
                        .unwrap_or("defaultname")       // Provide a default name in case of failure
                        .replace('_', "")               // Replace all underscores
                        .to_string();
                    
                    let _ = self.register_router::<WasmRouter>(router_id.clone(), router).await;
                }
            }
        }
    }

    // Register the router
    pub async fn register_router<T: Router>(&mut self, router_id: String, router: Box<dyn Router>) -> Result<(), String> {
        let tools = router.list_tools();
        let resources = router.list_resources();
        let prompts = router.list_prompts();
        let capabilities = router.capabilities().clone();
        let router_addr = RouterActor::new(Arc::new(router)).start();

        info!("Registering router {} at {:?}", router_id.clone(), router_addr.clone());
        //self.active_registry.register_router(router_id.clone(), router_addr.clone())?;
        let _ = self.active_registry
        .send(RegisterRouter { router_id: router_id.to_string(), router_addr: router_addr.clone(), capabilities: Some(capabilities)})
        .await
        .unwrap();

        if prompts.len() > 0 {
            self.list_prompts.do_send(AddPromptsRequest {
                router_id: router_id.clone(),
                prompts,
                router: router_addr.clone(),
            });
        }
        if tools.len() > 0 { 
            self.list_tools.do_send(AddToolsRequest {
                router_id: router_id.clone(),
                tools,
                router: router_addr.clone(),
            });
        }
        if resources.len() > 0 {
            self.list_resources.do_send(AddResourcesRequest {
                router_id: router_id.clone(),
                resources,
                router: router_addr.clone(),
            });
        }

        Ok(())
    }

    // Unregister the router
    pub async fn unregister_router(&mut self, router_id: &str) -> Result<(), String> {
        // Unregister the router
        let _ = self.active_registry
        .send(UnregisterRouter { router_id: router_id.to_string() })
        .await
        .unwrap();
        
        info!("Unregistered router: {}", router_id);
        Ok(())
    }

    pub async fn get_router(&self, action: String) -> Option<(Addr<RouterActor>, String)> {
        self.active_registry
        .send(GetRouter { router_id: action.clone(), _marker: std::marker::PhantomData })
        .await
        .unwrap()
    }

    pub fn get_registry(&self) -> Addr<ActorRouterRegistry> {
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
