use std::{future::Future, pin::Pin};
use mcp_ectors::router::{router::ResponseFuture, Router};
use mcp_spec::{handler::PromptError, prompt::{Prompt, PromptMessage, PromptMessageContent, PromptMessageRole}, protocol::{CallToolResult, GetPromptResult, InitializeResult, PromptsCapability, ReadResourceResult, ResourcesCapability, ServerCapabilities, ToolsCapability}, Annotations, Content::Text, Resource, ResourceContents::TextResourceContents, Role::User, TextContent, Tool};
use serde_json::Value;
use chrono::{DateTime, Utc, TimeZone};

/// A simple mock implementation of the Router trait that lets tests set
/// predetermined responses for specific methods.
#[derive(Clone)]
pub struct MockRouter {
    // Predefined response for the initialize method.
    pub _initialize_result: InitializeResult,
    // Similarly, you can add fields for other methods (tools, resources, prompts).
    pub tool_result: Vec<Tool>,
}

impl MockRouter {
    pub fn new(initialize_result: InitializeResult, tool_result: Vec<Tool>) -> Self {

        Self {
            _initialize_result: initialize_result,
            tool_result,
        }
    }
}

impl Router for MockRouter {
    fn name(&self) -> String {
        "MockRouter".to_string()
    }

    fn instructions(&self) -> String {
        "Mock instructions".to_string()
    }

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolsCapability{ list_changed: Some(true) }),
            resources: Some(ResourcesCapability{ subscribe: Some(true), list_changed: Some(false)}),
            prompts: Some(PromptsCapability{ list_changed: Some(true) }),
        }
    }
    

    fn list_tools(&self) -> Vec<Tool> {
        self.tool_result.clone()
    }

    fn call_tool(
        &self,
        tool_name: &str,
        _arguments: Value,
    ) -> Pin<Box<(dyn Future<Output = Result<CallToolResult, mcp_spec::ToolError>> + 'static)>> {
        let tool_name_owned = tool_name.to_string();
        Box::pin(async move {
            if tool_name_owned == "tool1" {
                // Assume that for echo_tool, the tool echoes the message.
                let dt: DateTime<Utc> = Utc.with_ymd_and_hms(2222, 2, 22,0, 0, 0).unwrap();
                let message = TextContent{ 
                    text: "default message".to_string(), 
                    annotations: Some(Annotations{ 
                        audience: Some(vec![User]), 
                        priority: Some(1.0), 
                        timestamp: Some(dt),
                    }) };
                let result = CallToolResult{ content: vec![Text(message)], is_error: Some(false) };
                Ok(result)
            } else {
                let result = CallToolResult{ content: vec![], is_error: Some(true) };
                Ok(result)
            }
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        let dt: DateTime<Utc> = Utc.with_ymd_and_hms(2222, 2, 22,0, 0, 0).unwrap();
        vec![
            Resource {
                uri:"echo://fixedresource".to_string(),
                description: Some("A fixed echo resource".to_string()), 
                name:"resource_name".to_string(), 
                mime_type: "text".to_string(), 
                annotations: Some(Annotations{ 
                    audience: Some(vec![User]),
                    priority: Some(1.0), 
                    timestamp: Some(dt), 
                })}
        ]
    }

    fn read_resource(
        &self,
        uri: &str,
    ) -> Pin<Box<(dyn Future<Output = Result<ReadResourceResult, mcp_spec::handler::ResourceError>> + 'static)>> {
        let uri_owned = uri.to_string();
        Box::pin(async move {
            if uri_owned == "echo://fixedresource" {
                let cwd = TextResourceContents{ uri:uri_owned, mime_type: Some("text/plain".to_string()), text: "expected resource value".to_string() };
                let result = ReadResourceResult{ contents: vec![cwd] };
                Ok(result)
            } else {
                let result = ReadResourceResult{ contents: vec![] };
                Ok(result)
            }
        })
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        vec![
            Prompt {
                name:"dummy_prompt".to_string(),
                description:Some("A dummy prompt for testing".to_string()), 
                arguments: None,
                //Some(vec![PromptArgument{
                //    name: "dummy_prompt_argument".to_string(),
                //    description: Some("dummy_promot_description".to_string()),
                //    required:Some(true)}]),
             }
        ]
    }

    fn get_prompt(&self, prompt_name: &str) -> ResponseFuture<Result<GetPromptResult, PromptError>> {
        let prompt = prompt_name.to_string(); 
        Box::pin(async move {
            let result = GetPromptResult {
                description: None,
                messages: vec![PromptMessage{
                    content:PromptMessageContent::Text{text:"dummy prompt response".to_string()},
                    role:PromptMessageRole::User,
                }],
            };
            
            if prompt == "dummy_prompt" {
                Ok(result.clone())  // Return the result when the prompt matches
            } else {
                Err(PromptError::NotFound(prompt))  // Return the error when the prompt does not match
            }
        })
    }
}