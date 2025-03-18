use actix::ResponseFuture;
use serde_json::Value;

use crate::router::{router::CapabilitiesBuilder, router_registry::ROUTER_SEPERATOR, Router};
use mcp_spec::{handler::ResourceError, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, ReadResourceResult, ServerCapabilities}, Resource, ResourceContents::{self, TextResourceContents}, Tool, ToolError};

/// **A simple Hello World router**
#[derive(Clone)]
pub struct SystemRouter{
    resources: Vec<ResourceContents>,
}

impl SystemRouter {
    pub fn new() -> Self {
            let resources = vec![];
        Self {
            resources,
        }
    }
}

impl Router for SystemRouter {
    fn name(&self) -> String {
        "system".to_string()
    }

    fn instructions(&self) -> String {
        format!("This is the system router who offers information about what is installed in this server. To get a list do resources/read uri: system{}all",ROUTER_SEPERATOR)
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new()
            .with_resources(true, true)
            .build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![]
    }

    fn call_tool(
        &self,
        tool_name: &str,
        _arguments: Value,
    ) -> ResponseFuture<Result<CallToolResult, ToolError>> {
        let tool_name = tool_name.to_string();

        Box::pin(async move {
            Err(ToolError::NotFound(format!("Tool {} not found", tool_name)))
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![Resource{ 
            uri: "all".to_string(), 
            name: "all resources, prompts, tools,... registered in this mcp multi router server".to_string(), 
            description: Some("this gives a description of all the resources, prompts, tools,... which different routers offer that have been installed in this multi-router mcp server.".to_string()), 
            mime_type: "text/plain".to_string(), 
            annotations: None }]
    }

    fn read_resource(
        &self,
        uri: &str,
    ) -> ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        let mut resources = self.resources.clone();
        resources.push(TextResourceContents{
            uri: "all".to_string(),
            mime_type: Some("plain/text".to_string()),
            text: "This multi-router mcp server currently has two routers installed:\n
            counter_router: This server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.\n
            hellow_world: This server responds with a greeting, 'Hello {name}', where 'name' is the parameter passed.
            ".to_string()});
            let uri_clone = uri.to_string();
        Box::pin(async move { 
            match uri_clone.as_str() {
                "all" => {Ok(ReadResourceResult{ contents: resources })},
                name => Err(ResourceError::NotFound(format!("Resource {} not found", name))) 
            }
            
        })
    }
    
    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }
    
    fn get_prompt(&self, _prompt_name: &str) -> ResponseFuture<Result<GetPromptResult, ResourceError>> {

        let result = GetPromptResult{ description: None, messages: vec![] };
        Box::pin(async move {
            Ok(result)
        })
    }
}

