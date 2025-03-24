use exports::wasix::mcp::router::{Role::User, Annotations, ToolsCapability, ResourcesCapability, PromptsCapability, CallToolResult, Content::Text, GetPromptResult, Guest, McpResource, Prompt, PromptMessage, PromptMessageContent, PromptMessageRole, ReadResourceResult, ResourceContents, ResourceError, ServerCapabilities, TextContent, TextResourceContents, Tool, ToolError, Value};

wit_bindgen::generate!({
    // with: {
    //     "wasix:mcp/router@0.0.1": generate,
    // }
    path: "../../wit/world.wit",
    world: "mcp",
});


/// A simple mock implementation of the Router trait that lets tests set
/// predetermined responses for specific methods.
#[derive(Clone)]
pub struct WasmMockRouter;

impl Guest for WasmMockRouter {
    fn name() -> String {
        "MockRouter".to_string()
    }

    fn instructions() -> String {
        "Mock instructions".to_string()
    }

    fn capabilities() -> ServerCapabilities {
        ServerCapabilities {
            tools: Some(ToolsCapability{ list_changed: Some(true) }),
            resources: Some(ResourcesCapability{ subscribe: Some(true), list_changed: Some(false)}),
            prompts: Some(PromptsCapability{ list_changed: Some(true) }),
        }
    }
    

    fn list_tools() -> Vec<Tool> {
        vec![Tool {
            name: "tool1".to_string(),
            description: "Test tool 1".to_string(),
            input_schema: Value{key:"input schema 1".to_string(), data:"{}".to_string()},
        }]
    }

    fn call_tool(
        tool_name: String,
        _arguments: Value,
    ) -> Result<CallToolResult, ToolError> {


        if tool_name == "tool1" {
            // Assume that for echo_tool, the tool echoes the message.
            let message = TextContent{ 
                text: "default message".to_string(), 
                annotations: Some(Annotations{ 
                    audience: Some(vec![User]), 
                    priority: Some(1.0), 
                    timestamp: Some("now".to_string()),
                }) };
            let result = CallToolResult{ content: vec![Text(message)], is_error: Some(false) };
            Ok(result)
        } else {
            let result = CallToolResult{ content: vec![], is_error: Some(true) };
            Ok(result)
        }
        
    }

    fn list_resources() -> Vec<McpResource> {
        vec![
            McpResource {
                uri:"echo://fixedresource".to_string(),
                description: Some("A fixed echo resource".to_string()), 
                name:"resource_name".to_string(), 
                mime_type: "text".to_string(), 
                annotations: Some(Annotations{ 
                    audience: Some(vec![User]),
                    priority: Some(1.0), 
                    timestamp: Some("now".to_string()), 
                })}
        ]
    }

    fn read_resource(
        uri: String,
    ) -> Result<ReadResourceResult, ResourceError> {

        if uri == "echo://fixedresource" {
            let cwd = TextResourceContents{ uri, mime_type: Some("text/plain".to_string()), text: "expected resource value".to_string() };
            let result = ReadResourceResult{ contents: vec![ResourceContents::Text(cwd)] };
            Ok(result)
        } else {
            let result = ReadResourceResult{ contents: vec![] };
            Ok(result)
        }
        
    }

    fn list_prompts() -> Vec<Prompt> {
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

    fn get_prompt(prompt_name: String) -> Result<GetPromptResult, ResourceError> {
        let result = GetPromptResult {
            description: None,
            messages: vec![PromptMessage{
                role:PromptMessageRole::User,
                content:PromptMessageContent::Text(TextContent{
                    text:"dummy prompt response".to_string(),
                    annotations: None,
                }
            )}],
        };
        
        if prompt_name == "dummy_prompt" {
            Ok(result.clone())  // Return the result when the prompt matches
        } else {
            Err(ResourceError::NotFound(prompt_name))  // Return the error when the prompt does not match
        }
        
    }
}

export!(WasmMockRouter);