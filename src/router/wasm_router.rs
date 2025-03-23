use std::{collections::VecDeque, future::Future, pin::Pin, sync::Arc};
use mcp_spec::{handler::ResourceError, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, PromptsCapability, ReadResourceResult, ResourcesCapability, ServerCapabilities, ToolsCapability}, Resource, Tool, ToolError};
use serde_json::Value;
use tokio::sync::Mutex;
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime::{component::{bindgen, Component, Linker}, AsContext, AsContextMut, Config, Engine, Store};

use super::Router;
pub type ResponseFuture<I> = Pin<Box<dyn Future<Output = I>>>;

bindgen!({
    world: "mcp",
});


pub struct WasmRouter {
    store_pool: Arc<Mutex<VecDeque<Store<MyState>>>>,
    mcp: Mcp,
}

impl WasmRouter {
    pub fn new(wasm_path: &str, pool_size: usize) -> Result<Self, anyhow::Error> {
        let file = wasm_path;
        let mut config = Config::default();
        config.async_support(true);

        // Create a Wasmtime engine and store
        let engine = Engine::new(&config).expect("engine could not be created");
        let wasi = WasiCtxBuilder::new().build();
        let state = MyState {
            ctx: wasi,
            table: ResourceTable::new(),
        };
        let mut store = Store::new(&engine, state);
        let component = Component::from_file(&engine, file).expect(format!("wasm file {} could not be read",file.clone()).as_str());
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_async::<MyState>(&mut linker).expect("Could not add wasi to wasm router");

        // Instantiate the MCP router from the wasm component
        let router = Mcp::instantiate(&mut store, &component, &linker)
            .map_err(|err| Box::new(err) as Box<anyhow::Error>).expect(format!("Could not instantiate wasm router: {}",file.clone()).as_str());
        
        // Build the pool – you might clone or re-instantiate stores as needed.
        let mut pool = VecDeque::new();
        for _ in 0..pool_size {
            let wasi = WasiCtxBuilder::new().build();
            let store = Store::new(&engine, MyState {
                ctx: wasi,
                table: ResourceTable::new(),
            });
            pool.push_back(store);
        }

        Ok(WasmRouter { store_pool: Arc::new(Mutex::new(pool)), mcp: router })
    }

    fn get_mcp(&self) -> &Mcp {
        &self.mcp
    }

    // Helper to get a store from the pool
    async fn get_store(&self) -> Option<Store<MyState>> {
        self.store_pool.lock().await.pop_front()
    }
    
    // And a helper to return the store to the pool after use.
    async fn return_store(&self, store: Store<MyState>) {
        self.store_pool.lock().await.push_back(store);
    }
}

impl WasiView for MyState
{
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}
impl IoView for MyState{
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl Router for WasmRouter {
    fn name(&self) -> String {
        //let mut guard = self.store.blocking_lock();
        //let store: &mut Store<MyState> = &mut *guard;

        let store = self.get_store().await;
        let name = self.get_mcp().wasix_mcp_router().call_name(store).unwrap();
        self.return_store(store);
        name
    }

    fn instructions(&self) -> String {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let instructions = self.get_mcp().wasix_mcp_router().call_instructions(store).unwrap();
        instructions
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolsCapability {
                list_changed: Some(true), // Example
            }),
            prompts: Some(PromptsCapability {
                list_changed: Some(true), // Example
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(true), // Example
                list_changed: Some(true), // Example
            }),
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let tools = self.get_mcp().wasix_mcp_router().call_list_tools(store).unwrap();
        tools
    }

    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> ResponseFuture<Result<CallToolResult, ToolError>> {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let result = self
            .get_mcp()
            .wasix_mcp_router()
            .call_call_tool(store, tool_name, &arguments)
            .unwrap();
        Box::pin(async { result })
    }
    

    fn list_resources(&self) -> Vec<Resource> {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let resources = self.get_mcp().wasix_mcp_router().call_list_resources(store).unwrap();
        resources
    }

    fn read_resource(
        &self,
        uri: &str,
    ) -> ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let result = self.get_mcp().wasix_mcp_router().call_read_resource(store, uri).unwrap();
        Box::pin(async { result })
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let prompts = self.get_mcp().wasix_mcp_router().call_list_prompts(store).unwrap();
        let mcp_prompts: Vec<mcp_spec::prompt::Prompt> = prompts.into();
        mcp_prompts
    }

    fn get_prompt(&self, prompt_name: &str) -> ResponseFuture<Result<GetPromptResult, ResourceError>> {
        let mut guard = self.store.blocking_lock();
        let store: &mut Store<MyState> = &mut *guard;
        let result = self
            .get_mcp()
            .wasix_mcp_router()
            .call_get_prompt(store, prompt_name)
            .unwrap();
        Box::pin(async { result })
    }
}

// Assuming MyState and other required structs (like Mcp, ResourceTable, etc.) are defined somewhere


pub struct MyState{
    ctx: WasiCtx,
    table: ResourceTable,
}




