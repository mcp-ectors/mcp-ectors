use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::router::{router::{CapabilitiesBuilder, ResponseFuture}, Router};
use mcp_spec::{handler::{PromptError, ResourceError}, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, ReadResourceResult, ServerCapabilities}, Content, Resource, ResourceContents::TextResourceContents, Tool, ToolError};

/// **A simple counter router that implements `RouterHandler`**
#[derive(Clone)]
pub struct CounterRouter {
    counter: Arc<Mutex<i32>>,
}

impl CounterRouter {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }

    async fn increment(&self) -> Result<i32, ()> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(*counter)
    }

    async fn decrement(&self) -> Result<i32, ()> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(*counter)
    }

    async fn get_value(&self) -> Result<i32, ()> {
        let counter = self.counter.lock().await;
        Ok(*counter)
    }

    fn create_resource_text(&self, uri: &str, name: &str) -> Resource {
        Resource::new(uri, Some("text/plain".to_string()), Some(name.to_string())).unwrap()
    }

}


impl Router for CounterRouter {
    fn name(&self) -> String {
        "counter".to_string()
    }

    fn instructions(&self) -> String {
        "This server provides a counter tool that can increment and decrement values. The counter starts at 0 and can be modified using the 'increment' and 'decrement' tools. Use 'get_value' to check the current count.".to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        CapabilitiesBuilder::new()
            .with_tools(false)
            .with_resources(false, false)
            .build()
    }

    fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new(
                "increment".to_string(),
                "Increment the counter by 1".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
            Tool::new(
                "decrement".to_string(),
                "Decrement the counter by 1".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
            Tool::new(
                "get_value".to_string(),
                "Get the current counter value".to_string(),
                serde_json::json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            ),
        ]
    }

    fn call_tool(
        &self,
        tool_name: &str,
        _arguments: Value,
    ) -> ResponseFuture<Result<CallToolResult, ToolError>> {
        let this = self.clone();
        let tool_name = tool_name.to_string();

        Box::pin(async move {
            match tool_name.as_str() {
                "increment" => {
                    let value = this.increment().await.expect("increment does not work");
                    let result = CallToolResult{ content: vec![Content::text(value.to_string())], is_error: Some(false) };
                    Ok(result)
                }
                "decrement" => {
                    let value = this.decrement().await.expect("decrement does not work");
                    let result = CallToolResult{ content: vec![Content::text(value.to_string())], is_error: Some(false) };
                    Ok(result)
                }
                "get_value" => {
                    let value = this.get_value().await.expect("get value does not work");
                    let result = CallToolResult{ content: vec![Content::text(value.to_string())], is_error: Some(false) };
                    Ok(result)
                }
                _ => Err(ToolError::NotFound(format!("Tool {} not found", tool_name))),
            }
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        vec![
            self.create_resource_text("str://///Users/maarten/ai/test", "cwd"),
            self.create_resource_text("memo://insights", "memo-name"),
        ]
    }

    fn read_resource(
        &self,
        uri: &str,
    ) -> ResponseFuture<Result<ReadResourceResult, ResourceError>> {
        let uri = uri.to_string();
        Box::pin(async move {
            match uri.as_str() {
                "str://///Users/maarten/ai/test/" => {
                    let cwd = TextResourceContents{ uri, mime_type: Some("text/plain".to_string()), text: "/Users/maarten/ai/test/".to_string() };
                    let result = ReadResourceResult{ contents: vec![cwd] };
                    Ok(result)
                }
                "memo://insights" => {
                    let cwd = TextResourceContents{ uri, mime_type: Some("text/plain".to_string()), text: "Business Intelligence Memo\n\nAnalysis has revealed 5 key insights ...".to_string() };
                    let result = ReadResourceResult{ contents: vec![cwd] };
                    Ok(result)
                }
                _ => Err(ResourceError::NotFound(format!(
                    "Resource {} not found",
                    uri
                ))),
            }
        })
    }
    
    fn list_prompts(&self) -> Vec<Prompt> {
        vec![]
    }
    
    fn get_prompt(&self, _prompt_name: &str) -> ResponseFuture<Result<GetPromptResult, PromptError>> {

        let result = GetPromptResult{ description: None, messages: vec![] };
        Box::pin(async move {
            Ok(result)
        })
    }
}
