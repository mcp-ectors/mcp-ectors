use std::{future::Future, pin::Pin};
use exports::wasix::mcp::router::Guest;
use mcp_spec::{handler::ResourceError, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, PromptsCapability, ReadResourceResult, ResourcesCapability, ServerCapabilities, ToolsCapability}, Resource, Tool, ToolError};
use serde_json::Value;
use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder};
use wasmtime::{component::{bindgen, Component, Linker}, Config, Engine, Store};

use super::Router;
pub type ResponseFuture<I> = Pin<Box<dyn Future<Output = I>>>;

bindgen!({
    world: "mcp",
});


pub struct WasmRouter {
    store: Store<MyState>,
    mcp: Mcp,
}

impl WasmRouter {
    pub fn new(wasm_path: &str) -> Result<Self, anyhow::Error> {
        let file = wasm_path;
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
        let component = Component::from_file(&engine, file).expect(format!("wasm file {} could not be read",file.clone()).as_str());
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker_sync(&mut linker).expect("Could not add wasi to wasm router");

        
        // Instantiate the MCP router from the wasm component
        let router = Mcp::instantiate(&mut store, &component, &linker)
            .map_err(|err| Box::new(err) as Box<anyhow::Error>).expect(format!("Could not instantiate wasm router: {}",file.clone()).as_str());
        
        Ok(WasmRouter { store, mcp: router })
    }

    fn get_mcp(&self) -> &Mcp {
        &self.mcp
    }
}

impl Router for WasmRouter {
    fn name(& self) -> String {
        let name = self.get_mcp().wasix_mcp_router().call_name(&mut self.store).unwrap();
        name
    }

    fn instructions(&self) -> String {
        let instructions = self.get_mcp().wasix_mcp_router().call_instructions(&mut self.store).unwrap();
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
        let tools = self.get_mcp().wasix_mcp_router().call_list_tools(&mut self.store).unwrap();
        tools
    }

    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> ResponseFuture<Result<CallToolResult, ToolError>> {
        let result = self
            .get_mcp()
            .wasix_mcp_router()
            .call_call_tool(&mut self.store, tool_name, &arguments)
            .unwrap();
        Box::pin(async { result })
    }

    fn list_resources(&self) -> Vec<Resource> {
        let resources = self.get_mcp().wasix_mcp_router().call_list_resources(&mut self.store).unwrap();
        resources
    }

    fn read_resource(
        &self,
        uri: &str,
    ) -> ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        let result = self.get_mcp().wasix_mcp_router().call_read_resource(&mut self.store, uri).unwrap();
        Box::pin(async { result })
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        let prompts = self.get_mcp().wasix_mcp_router().call_list_prompts(&mut self.store).unwrap();
        let mcp_prompts: Vec<mcp_spec::prompt::Prompt> = prompts.into();
        mcp_prompts
    }

    fn get_prompt(&self, prompt_name: &str) -> ResponseFuture<Result<GetPromptResult, ResourceError>> {
        let result = self
            .get_mcp()
            .wasix_mcp_router()
            .call_get_prompt(&mut self.store, prompt_name)
            .unwrap();
        Box::pin(async { result })
    }
}

// Assuming MyState and other required structs (like Mcp, ResourceTable, etc.) are defined somewhere

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

// Assuming wasix::mcp::router::Tool is generated from WIT, and mcp_spec::protocol::Tool is the target type

impl From<wasix::mcp::router::Tool> for mcp_spec::protocol::Tool {
    fn from(wasix_tool: wasix::mcp::router::Tool) -> Self {
        mcp_spec::protocol::Tool {
            name: wasix_tool.name,
            description: wasix_tool.description,
            input_schema: wasix_tool.input_schema.into(), // Assuming value needs conversion
        }
    }
}

// Assuming wasix::mcp::router::Value is generated from WIT, and mcp_spec::protocol::Value is the target type

impl From<wasix::mcp::router::Value> for mcp_spec::protocol::Value {
    fn from(wasix_value: wasix::mcp::router::Value) -> Self {
        mcp_spec::protocol::Value {
            key: wasix_value.key,
            data: wasix_value.data,
        }
    }
}


impl From<wasix::mcp::router::CallToolResult> for mcp_spec::protocol::CallToolResult {
    fn from(wasix_result: wasix::mcp::router::CallToolResult) -> Self {
        mcp_spec::protocol::CallToolResult {
            content: wasix_result.content.into_iter().map(|c| c.into()).collect(),
            is_error: wasix_result.is_error,
        }
    }
}

impl From<wasix::mcp::router::McpResource> for mcp_spec::protocol::Resource {
    fn from(wasix_resource: wasix::mcp::router::McpResource) -> Self {
        mcp_spec::protocol::Resource {
            uri: wasix_resource.uri,
            name: wasix_resource.name,
            description: wasix_resource.description,
            mime_type: wasix_resource.mime_type,
            annotations: wasix_resource.annotations,
        }
    }
}

impl From<wasix::mcp::router::Prompt> for mcp_spec::protocol::Prompt {
    fn from(wasix_prompt: wasix::mcp::router::Prompt) -> Self {
        mcp_spec::protocol::Prompt {
            name: wasix_prompt.name,
            description: wasix_prompt.description,
            arguments: wasix_prompt.arguments.into_iter().map(|arg| arg.into()).collect(),
        }
    }
}


