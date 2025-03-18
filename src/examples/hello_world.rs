use actix::prelude::*;
use serde_json::Value;

use crate::router::{router::CapabilitiesBuilder, Router};
use mcp_spec::{handler::ResourceError, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, ReadResourceResult, ServerCapabilities}, Content, Resource, Tool, ToolError};

/// **A simple Hello World router**
#[derive(Clone)]
pub struct HelloWorldRouter;

impl HelloWorldRouter {
    pub fn new() -> Self {
        Self {}
    }
}

/// **Actix Actor implementation for HelloWorldRouter**
impl Actor for HelloWorldRouter {
    type Context = Context<Self>;
}

impl Router for HelloWorldRouter {
    fn name(&self) -> String {
        "hello_world".to_string()
    }

    fn instructions(&self) -> String {
        "This server responds with a greeting, 'Hello {name}', where 'name' is the parameter passed.".to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new()
            .with_tools(true)
            .with_prompts(false)
            .with_resources(false, false)
            .build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "greet".to_string(),
                "Responds with a greeting 'Hello {name}'".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" }
                    },
                    "required": ["name"]
                }),
            ),
        ]
    }

    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> ResponseFuture<Result<CallToolResult, ToolError>> {
        let tool_name = tool_name.to_string();

        Box::pin(async move {
            match tool_name.as_str() {
                "greet" => {
                    // Expect the "name" parameter in the arguments
                    if let Some(name) = arguments.get("name").and_then(|v| v.as_str()) {
                        let greeting = format!("Hello {}", name);
                        let result = CallToolResult{ content: vec![Content::text(greeting)], is_error: Some(false) };
                        Ok(result)
                    } else {
                        Err(ToolError::InvalidParameters("Missing or invalid 'name' parameter.".to_string()))
                    }
                }
                _ => Err(ToolError::NotFound(format!("Tool {} not found", tool_name))),
            }
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![]
    }

    fn read_resource(
        &self,
        _uri: &str,
    ) -> ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        Box::pin(async { Err(ResourceError::NotFound("Resource not found".to_string())) })
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

