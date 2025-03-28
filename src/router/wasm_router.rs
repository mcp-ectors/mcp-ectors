use std::{future::Future, pin::Pin, sync::Arc, thread};

use exports::wasix;
use mcp_spec::{ handler::{PromptError, ResourceError}, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, ReadResourceResult, ServerCapabilities}, Resource, Tool, ToolError};
use serde_json::Value as JsonValue;
use tracing::error;
use std::sync::mpsc::{self, Sender, Receiver};
use wasmtime_wasi::{IoView, ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime::{component::{bindgen, Component, Linker}, Config, Engine, Store};
use std::convert::Into;

use super::{wasix_mcp::json_to_value, Router};
pub type ResponseFuture<I> = Pin<Box<dyn Future<Output = I>>>;

bindgen!({
    world: "mcp",
});



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
    ListResources,
    ListPrompts,
    ReadResource(String),
    GetPrompt(String),
    CallTool(String, JsonValue),
    Capabilities,
    // Add other request types as needed.
}

/// Define the possible responses from the WASM thread.
#[allow(dead_code)]
enum WasmResponse {
    Name(String),
    Instructions(String),
    Tools(Vec<Tool>),
    Prompts(Vec<Prompt>),
    GetPromptResult(GetPromptResult),
    CallToolResult(CallToolResult),
    Resources(Vec<Resource>),
    ReadResource(ReadResourceResult),
    Capabilities(ServerCapabilities),
    RetToolError(ToolError),
    RetResourceError(ResourceError),
    RetPromptError(PromptError),
    Error(String),
}

/// A handle that lets callers send synchronous requests to the dedicated WASM thread.
pub struct WasmRouterHandle {
    request_tx: Sender<(WasmRequest, Sender<WasmResponse>)>,
}

impl WasmRouterHandle {
    fn send_request(&self, request: WasmRequest) -> Result<WasmResponse, String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        self.request_tx
            .send((request, resp_tx))
            .map_err(|e| format!("Send error: {}", e))?;
        match resp_rx.recv() {
            Ok(response) => match response {
                WasmResponse::Error(err) => Err(err),
                _ => Ok(response),
            },
            Err(_) => Err("Unexpected response".into()),
        }
    }

    pub fn get_name(&self) -> Result<String, String> {
        match self.send_request(WasmRequest::GetName)? {
            WasmResponse::Name(name) => Ok(name),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }
    
    pub fn get_instructions(&self) -> Result<String, String> {
        match self.send_request(WasmRequest::GetInstructions)? {
            WasmResponse::Instructions(instr) => Ok(instr),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }
    
    pub fn list_tools(&self) -> Result<Vec<Tool>, String> {
        match self.send_request(WasmRequest::ListTools)? {
            WasmResponse::Tools(tools) => Ok(tools),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }

    pub fn list_resources(&self) -> Result<Vec<Resource>, String> {
        match self.send_request(WasmRequest::ListResources)? {
            WasmResponse::Resources(resources) => Ok(resources),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }

    pub fn list_prompts(&self) -> Result<Vec<Prompt>, String> {
        match self.send_request(WasmRequest::ListPrompts)? {
            WasmResponse::Prompts(prompts) => Ok(prompts),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }

    pub fn get_prompt(&self, prompt_name: &str) -> Result<GetPromptResult, String> {
        match self.send_request(WasmRequest::GetPrompt(prompt_name.to_string()))? {
            WasmResponse::GetPromptResult(prompt) => Ok(prompt),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }

    pub fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, String> {
        match self.send_request(WasmRequest::ReadResource(uri.to_string()))? {
            WasmResponse::ReadResource(resource) => Ok(resource),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }

    pub fn call_tool(&self, tool_name: &str, arguments: JsonValue) -> Result<CallToolResult, String> {
        match self.send_request(WasmRequest::CallTool(tool_name.to_string(), arguments))? {
            WasmResponse::CallToolResult(result) => Ok(result),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
        }
    }

    pub fn capabilities(&self) -> Result<ServerCapabilities, String> {
        match self.send_request(WasmRequest::Capabilities)? {
            WasmResponse::Capabilities(capabilities) => Ok(capabilities),
            WasmResponse::Error(err) => Err(err),
            _ => Err("Unexpected response type".into()),
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
                WasmRequest::ListPrompts => {
                    match router.wasix_mcp_router().call_list_prompts(&mut store) {
                        Ok(prompts) => {
                            // If the call returns a nested vector, flatten it.
                            let mcp_prompts: Vec<mcp_spec::prompt::Prompt> = prompts
                                .into_iter()
                                .map(|prompt| mcp_spec::prompt::Prompt::from(prompt))
                                .collect();
                            WasmResponse::Prompts(mcp_prompts)
                        },
                        Err(e) => WasmResponse::Error(e.to_string()),
                    }
                },
                WasmRequest::ListResources => {
                    match router.wasix_mcp_router().call_list_resources(&mut store) {
                        Ok(resources) =>{
                            let mcp_resources: Vec<mcp_spec::resource::Resource> = resources
                            .into_iter()
                            .map(|resource| mcp_spec::resource::Resource::from(resource))
                            .collect();
                            WasmResponse::Resources(mcp_resources)
                        },
                        Err(e) => WasmResponse::Error(e.to_string()),
                    }
                },
                WasmRequest::ReadResource(uri) => {
                    match router.wasix_mcp_router().call_read_resource(&mut store, &uri) {
                        Ok(resource_result) => {
                            match resource_result {
                                // Correctly match ReadResourceResult
                                Ok(wasix::mcp::router::ReadResourceResult { contents }) => {
                                    let resource_contents: Vec<mcp_spec::resource::ResourceContents> = contents
                                    .into_iter()
                                    .map(|resource| mcp_spec::resource::ResourceContents::from(resource))
                                    .collect();
                                    let resource_result = ReadResourceResult{
                                        contents: resource_contents,
                                    };
                                    WasmResponse::ReadResource(resource_result)
                                },
                                // Handle the ResourceError
                                Err(wasix::mcp::router::ResourceError::ExecutionError(error)) => {
                                    WasmResponse::RetResourceError(mcp_spec::handler::ResourceError::ExecutionError(error))
                                },
                                Err(wasix::mcp::router::ResourceError::NotFound(error)) => {
                                    WasmResponse::RetResourceError(mcp_spec::handler::ResourceError::NotFound(error))
                                },
                            }
                        },
                        Err(e) => {
                            // Handle any other errors from call_read_resource
                            WasmResponse::Error(format!("Failed to read resource: {}", e))
                        },
                    }
                },                
                WasmRequest::GetPrompt(name) => {
                    match router.wasix_mcp_router().call_get_prompt(&mut store, &name) {
                        Ok(prompt_result) => {
                            match prompt_result {
                                // Correctly match ReadResourceResult
                                Ok(wasix::mcp::router::GetPromptResult{description, messages}) => {
                                    let prompt_messages: Vec<mcp_spec::prompt::PromptMessage> = messages
                                        .into_iter()
                                        .map(|resource| mcp_spec::prompt::PromptMessage::from(resource)) // Fix to correctly map each `resource`
                                        .collect();
                                
                                    let prompt = mcp_spec::protocol::GetPromptResult {
                                        description,
                                        messages: prompt_messages,
                                    };
                                
                                    WasmResponse::GetPromptResult(prompt)
                                },
                                // Handle the ResourceError
                                Err(wasix::mcp::router::PromptError::InvalidParameters(error)) => {
                                    WasmResponse::RetPromptError(mcp_spec::handler::PromptError::InvalidParameters(error))
                                },
                                Err(wasix::mcp::router::PromptError::NotFound(error)) => {
                                    WasmResponse::RetPromptError(mcp_spec::handler::PromptError::NotFound(error))
                                },
                                Err(wasix::mcp::router::PromptError::InternalError(error)) => {
                                    WasmResponse::RetPromptError(mcp_spec::handler::PromptError::InternalError(error))
                                },
                            }
                        },
                        Err(e) => {
                            // Handle any other errors from call_read_resource
                            WasmResponse::Error(format!("Failed to read resource: {}", e))
                        },
                    }
                },
                WasmRequest::CallTool(name, value) => {
                    let mcp_value = json_to_value(value).or(Some(crate::router::wasm_router::wasix::mcp::router::Value{key:"".to_string(),data:"".to_string()}));
                    match router.wasix_mcp_router()
                        .call_call_tool(&mut store,
                        name.as_str(), 
                        &mcp_value.unwrap()) 
                        {
                            Ok(tool) => match tool {
                                Ok(wasix::mcp::router::CallToolResult{content, is_error}) => {
                                    let contents: Vec<mcp_spec::Content> = content
                                        .into_iter()
                                        .map(|item| mcp_spec::Content::from(item)) // Fix to correctly map each `resource`
                                        .collect();
                                    WasmResponse::CallToolResult(CallToolResult { content:contents, is_error})
                                },
                                Err(wasix::mcp::router::ToolError::ExecutionError(error)) => WasmResponse::RetToolError(ToolError::ExecutionError(error)), 
                                Err(wasix::mcp::router::ToolError::InvalidParameters(error)) => WasmResponse::RetToolError(ToolError::InvalidParameters(error)),
                                Err(wasix::mcp::router::ToolError::NotFound(error)) => WasmResponse::RetToolError(ToolError::NotFound(error)),
                                Err(wasix::mcp::router::ToolError::SchemaError(error)) => WasmResponse::RetToolError(ToolError::SchemaError(error)),
                                
                            },
                            Err(e) => WasmResponse::Error(e.to_string()),
                
                        }
                },
                WasmRequest::Capabilities => {
                    match router.wasix_mcp_router()
                        .call_capabilities(&mut store) 
                            {
                                Ok(capab) => {
                                        let mcp_cap = mcp_spec::protocol::ServerCapabilities::from(capab);
                                        WasmResponse::Capabilities(mcp_cap)
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
        match self.handle.capabilities() {
            Ok(caps) => caps,
            Err(err) => {
                error!("Error in server capabilities: {}", err);
                ServerCapabilities{prompts:None,resources:None,tools:None}
            }
        }
    }
    
    fn call_tool(
        &self,
        tool_name: &str,
        arguments: JsonValue,
    ) -> super::router::ResponseFuture<Result<CallToolResult, ToolError>> {
        match self.handle.call_tool(tool_name, arguments.clone()) {
            Ok(tool_result) => {
                Box::pin(async move {
                    Ok(tool_result)
                })
            },
            Err(err) => {
                error!("Error in call tool to {} with {}: {}", tool_name, arguments, err);
                Box::pin(async move {
                    Err(ToolError::ExecutionError(err))
                })
            }
        }
    }
    
    fn list_resources(&self) -> Vec<Resource> {
        match self.handle.list_resources() {
            Ok(resources) => resources,
            Err(err) => {
                error!("Error in list_resources: {}", err);
                vec![]
            }
        }
    }
    
    fn read_resource(
        &self,
        uri: &str,
    ) -> super::router::ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        match self.handle.read_resource(uri) {
            Ok(resource) => {
                Box::pin(async move {
                    Ok(resource)
                })
            },
            Err(err) => {
                error!("Error in reading resource for {}: {}", uri, err);
                Box::pin(async move {
                    Err(ResourceError::ExecutionError(err))
                })
            }
        }
    }
    
    fn list_prompts(&self) -> Vec<Prompt> {
        match self.handle.list_prompts() {
            Ok(prompts) => prompts,
            Err(err) => {
                error!("Error in list_prompts: {}", err);
                vec![]
            }
        }
    }
    
    fn get_prompt(&self, prompt_name: &str) -> super::router::ResponseFuture<Result<GetPromptResult, PromptError>> {
        match self.handle.get_prompt(prompt_name) {
            Ok(prompt) => {
                Box::pin(async move {
                    Ok(prompt)
                })
            },
            Err(err) => {
                error!("Error in getting prompt for {}: {}", prompt_name, err);
                Box::pin(async move {
                    Err(PromptError::InternalError(err))
                })
            }
        }
    }
}
