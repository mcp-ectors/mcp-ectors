

use std::sync::Arc;

use actix::{Actor,Context, Handler, ResponseFuture};

use mcp_spec::{handler::{PromptError, ResourceError}, prompt::Prompt, protocol::{CallToolResult, ErrorData, GetPromptResult, JsonRpcError, JsonRpcResponse, ReadResourceResult}, Resource, Tool, ToolError};
use serde_json::{json, Value};
use tracing::error;


use crate::messages::{TransportRequest, JSONRPC_VERSION};

use super::Router;

// The `RouterActor` will wrap each `Router` and act as an actor
pub struct RouterActor
{
    router: Arc<Box<dyn Router>>, // The actual router instance
}

impl RouterActor
{
    pub fn new(router: Arc<Box<dyn Router>>) -> Self {
        Self { router }
    }
}

impl Actor for RouterActor
{
    type Context = Context<Self>;
}

// Delegate Router trait methods to the internal router instances in the actor
impl RouterActor
{
    pub async fn list_tools(&self) -> Vec<Tool> {
        self.router.list_tools()
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: Value) -> Result<CallToolResult, ToolError> {
        self.router.call_tool(tool_name, arguments).await
    }

    pub async fn list_resources(&self) -> Vec<Resource> {
        self.router.list_resources()
    }

    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult, ResourceError> {
        self.router.read_resource(uri).await
    }

    pub async fn list_prompts(&self) -> Vec<Prompt> {
        self.router.list_prompts()
    }

    pub async fn get_prompt(&self, prompt_name: &str) -> Result<GetPromptResult, PromptError> {
        self.router.get_prompt(prompt_name).await
    }
}

async fn handle_request(method: String, params: Value, router_clone: Arc<Box<dyn Router>>, id: Option<u64>) -> Result<JsonRpcResponse, JsonRpcError>
{
    let result = match method.as_str() {
        "tools/call" => {
            if let Some(tool_name) = params.get("name").and_then(|v| v.as_str()) {
                let arguments = params.get("arguments").cloned().unwrap_or_default();
                let call_result = router_clone.call_tool(tool_name, arguments).await;
                match call_result {
                    Ok(content) => Ok(json!(content)),
                    Err(e) => {
                        error!("Failed to call tool: {:?}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: JSONRPC_VERSION.to_string(),
                            id,
                            result: None,
                            error: Some(ErrorData{
                                code: -32603, 
                                message: "Tool execution failed".to_string(),
                                data: Some(json!({
                                    "error": format!("{:?}", e)  // Provide the error details
                                })),
                            }),
                        };
                        Err(error_response)
                    }
                }
            } else {
                Err(JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id,
                    result: None,
                    error: Some(ErrorData {
                        code: -32602, // Invalid params error code
                        message: "Invalid parameters for call_tool".to_string(),
                        data: None,
                    }),
                })
            }
        },
        "tools/list" => {
            let tools = router_clone.list_tools(); // Call the list_tools method
            Ok(json!(tools))
        },
        
        "resources/list" => {
            let resources = router_clone.list_resources();
            Ok(json!(resources))
        },
        "resources/read" => {
            if let Some(uri) = params.get("uri").and_then(|v| v.as_str()) {
                let read_result = router_clone.read_resource(uri).await;
                match read_result {
                    Ok(content) => Ok(json!(content)),
                    Err(e) => {
                        error!("Failed to read resource: {:?}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: JSONRPC_VERSION.to_string(),
                            id,
                            result: None,
                            error: Some(ErrorData{
                                code: -32603, 
                                message: "Resource execution failed".to_string(),
                                data: Some(json!({
                                    "error": format!("{:?}", e)  // Provide the error details
                                })),
                            }),
                        };
                        Err(error_response)
                    }
                }
            } else {
                Err(JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id,
                    result: None,
                    error: Some(ErrorData {
                        code: -32602, // Invalid params error code
                        message: "Invalid parameters for read resource".to_string(),
                        data: None,
                    }),
                })
            }
        },
        "prompts/list" => {
            let prompts = router_clone.list_prompts();
            Ok(json!(prompts))
        },
        "prompts/get" => {
            if let Some(prompt_name) = params.get("name").and_then(|v| v.as_str()) {
                let prompt_result = router_clone.get_prompt(prompt_name).await;
                match prompt_result {
                    Ok(prompt) => Ok(json!(prompt)),
                    Err(e) => {
                        error!("Failed to call prompt: {:?}", e);
                        let error_response = JsonRpcResponse {
                            jsonrpc: JSONRPC_VERSION.to_string(),
                            id,
                            result: None,
                            error: Some(ErrorData{
                                code: -32603, 
                                message: "Prompt execution failed".to_string(),
                                data: Some(json!({
                                    "error": format!("{:?}", e)  // Provide the error details
                                })),
                            }),
                        };
                        Err(error_response)
                    }
                }
            } else {
                Err(JsonRpcResponse {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id,
                    result: None,
                    error: Some(ErrorData {
                        code: -32602, // Invalid params error code
                        message: "Invalid parameters for call_prompt".to_string(),
                        data: None,
                    }),
                })
            }
        },
        _ => Err(JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(ErrorData {
                code: -32602, // Invalid params error code
                message: "unsupported method".to_string(),
                data: None,
            }),
        })
    };

    // Create a JsonRpcResponse based on the result of the method
    match result {
        Ok(data) => Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(data),
            error: None,
        }),
        Err(_) => Ok(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(ErrorData{
                code: -32602,
                message: "Invalid params".to_string(),
                data: None,
            }),
        }),
    }
}

impl Handler<TransportRequest> for RouterActor
{
    type Result =  ResponseFuture<Result<JsonRpcResponse, JsonRpcError>>;



    fn handle(&mut self, msg: TransportRequest, _ctx: &mut Self::Context) -> Self::Result {
        // Assuming TransportRequest contains the method name and parameters for the router
        let method = msg.request.method.clone();
        let params = msg.request.params.unwrap();
        let router_clone = Arc::clone(&self.router);
        let id = msg.request.id;
        Box::pin(async move {
            handle_request(method, params, router_clone, id).await
            
        })
    }

        
    
}
