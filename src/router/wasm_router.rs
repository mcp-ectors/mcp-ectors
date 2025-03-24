use std::{future::Future, pin::Pin, sync::Arc, thread};
use exports::wasix;
use mcp_spec::{handler::ResourceError, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, ReadResourceResult, ServerCapabilities}, Resource, Tool, ToolError};
use serde_json::{Map, Value as JsonValue};
use tracing::error;
use std::sync::mpsc::{self, Sender, Receiver};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime::{component::{bindgen, Component, Linker}, Config, Engine, Store};
use std::convert::Into;

use super::Router;
pub type ResponseFuture<I> = Pin<Box<dyn Future<Output = I>>>;

bindgen!({
    world: "mcp",
});

/* 
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
*/

pub struct MyState{
    ctx: WasiCtx,
    table: ResourceTable,
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

/// Define the types of WASM requests.
enum WasmRequest {
    GetName,
    GetInstructions,
    ListTools,
    // Add other request types as needed.
}

/// Define the possible responses from the WASM thread.
enum WasmResponse {
    Name(String),
    Instructions(String),
    Tools(Vec<Tool>),
    Error(String),
}

/// A handle that lets callers send synchronous requests to the dedicated WASM thread.
pub struct WasmRouterHandle {
    request_tx: Sender<(WasmRequest, Sender<WasmResponse>)>,
}

impl WasmRouterHandle {
    pub fn get_name(&self) -> Result<String, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.request_tx
            .send((WasmRequest::GetName, resp_tx))
            .map_err(|e| format!("Send error: {}", e))?;
        match resp_rx.recv() {
            Ok(WasmResponse::Name(name)) => Ok(name),
            Ok(WasmResponse::Error(err)) => Err(err),
            _ => Err("Unexpected response".into()),
        }
    }
    
    pub fn get_instructions(&self) -> Result<String, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.request_tx
            .send((WasmRequest::GetInstructions, resp_tx))
            .map_err(|e| format!("Send error: {}", e))?;
        match resp_rx.recv() {
            Ok(WasmResponse::Instructions(instr)) => Ok(instr),
            Ok(WasmResponse::Error(err)) => Err(err),
            _ => Err("Unexpected response".into()),
        }
    }
    
    pub fn list_tools(&self) -> Result<Vec<Tool>, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.request_tx
            .send((WasmRequest::ListTools, resp_tx))
            .map_err(|e| format!("Send error: {}", e))?;
        match resp_rx.recv() {
            Ok(WasmResponse::Tools(tools)) => Ok(tools),
            Ok(WasmResponse::Error(err)) => Err(err),
            _ => Err("Unexpected response".into()),
        }
    }
}

/// Spawns a dedicated thread that owns the WASM instance and processes requests.
/// In your real code you’d initialize the WASM engine, store, component, etc. here.
pub fn spawn_wasm_router(wasm_path: &str) -> WasmRouterHandle {
    let (req_tx, req_rx): (
        Sender<(WasmRequest, Sender<WasmResponse>)>,
        Receiver<(WasmRequest, Sender<WasmResponse>)>,
    ) = mpsc::channel();
    
    let file = wasm_path.to_owned();    
    thread::spawn(move || {

        // --- Initialization ---
        let mut config = Config::default();
        config.async_support(false);

        // Create a Wasmtime engine and store
        let engine = Engine::new(&config).expect("engine could not be created");
        let wasi = WasiCtxBuilder::new().build();
        let state = MyState {
            ctx: wasi,
            table: ResourceTable::new(),
        };
        let mut store = Store::new(&engine, state);
        let component = Component::from_file(&engine, file.clone()).expect(format!("wasm file {} could not be read",file).as_str());
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_sync::<MyState>(&mut linker).expect("Could not add wasi to wasm router");

        // Instantiate the MCP router from the wasm component
        let router = Mcp::instantiate(&mut store, &component, &linker)
            .map_err(|err| Box::new(err) as Box<anyhow::Error>).expect(format!("Could not instantiate wasm router: {}",file).as_str());

        
        // --- Event Loop ---
        // Process incoming requests one at a time on this dedicated thread.
        for (request, resp_tx) in req_rx {
            let response = match request {
                WasmRequest::GetName => {
                    match router.wasix_mcp_router().call_name(&mut store) {
                        Ok(name) => WasmResponse::Name(name),
                        Err(e) => WasmResponse::Error(e.to_string()),
                    }
                },
                WasmRequest::GetInstructions => {
                    match router.wasix_mcp_router().call_instructions(&mut store) {
                        Ok(instr) => WasmResponse::Instructions(instr),
                        Err(e) => WasmResponse::Error(e.to_string()),
                    }
                },
                WasmRequest::ListTools => {
                    match router.wasix_mcp_router().call_list_tools(&mut store) {
                        Ok(tools) => {
                            // If the call returns a nested vector, flatten it.
                            let mcp_tools: Vec<mcp_spec::Tool> = tools
                                .into_iter()
                                .map(|tool| mcp_spec::Tool::from(tool))
                                .collect();
                            WasmResponse::Tools(mcp_tools)
                        },
                        Err(e) => WasmResponse::Error(e.to_string()),
                    }
                },
            };
            let _ = resp_tx.send(response);
        }
    });
    
    WasmRouterHandle {
        request_tx: req_tx,
    }
}

/// An Actix-compatible router implementation that wraps the WasmRouterHandle.
/// If a WASM call fails, it logs the error and returns an empty string (or empty vector).
pub struct WasmRouter {
    handle: Arc<WasmRouterHandle>,
}

impl WasmRouter {
    pub fn new(handle: WasmRouterHandle) -> Self {
        Self {
            handle: Arc::new(handle),
        }
    }
}

impl Router for WasmRouter {
    fn name(&self) -> String {
        match self.handle.get_name() {
            Ok(name) => name,
            Err(err) => {
                error!("Error in name: {}", err);
                "".into()
            }
        }
    }
    
    fn instructions(&self) -> String {
        match self.handle.get_instructions() {
            Ok(instr) => instr,
            Err(err) => {
                error!("Error in instructions: {}", err);
                "".into()
            }
        }
    }
    
    fn list_tools(&self) -> Vec<Tool> {
        match self.handle.list_tools() {
            Ok(tools) => tools,
            Err(err) => {
                error!("Error in list_tools: {}", err);
                vec![]
            }
        }
    }
    
    fn capabilities(&self) -> ServerCapabilities {
        todo!()
    }
    
    fn call_tool(
        &self,
        _tool_name: &str,
        _arguments: JsonValue,
    ) -> super::router::ResponseFuture<Result<CallToolResult, ToolError>> {
        todo!()
    }
    
    fn list_resources(&self) -> Vec<Resource> {
        todo!()
    }
    
    fn read_resource(
        &self,
        _uri: &str,
    ) -> super::router::ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        todo!()
    }
    
    fn list_prompts(&self) -> Vec<Prompt> {
        todo!()
    }
    
    fn get_prompt(&self, _prompt_name: &str) -> super::router::ResponseFuture<Result<GetPromptResult, ResourceError>> {
        todo!()
    }
}

impl From<wasix::mcp::router::Tool> for mcp_spec::Tool {
    fn from(tool: wasix::mcp::router::Tool) -> Self {
        mcp_spec::Tool {
            name: tool.name,
            description: tool.description,
            input_schema: value_to_json(tool.input_schema),
            // Convert additional fields here as needed.
        }
    }
}

fn value_to_json(val: wasix::mcp::router::Value) -> JsonValue {
    // Attempt to parse the data as JSON.
    // If parsing fails, fallback to using the string value.
    let parsed: JsonValue = serde_json::from_str(&val.data).unwrap_or(JsonValue::String(val.data));
    // Create a JSON object with the key/value.
    let mut map = Map::new();
    map.insert(val.key, parsed);
    JsonValue::Object(map)
}