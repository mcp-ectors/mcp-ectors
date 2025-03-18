use actix::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use mcp_spec::protocol::JsonRpcResponse;
use mcp_spec::protocol::JsonRpcRequest;
pub const JSONRPC_VERSION: &str = "2.0";

/// A unified type for handling all JSON-RPC requests and notifications.
#[derive(Message)]
#[rtype(result = "Result<JsonRpcResponse, ()>")]
pub enum JsonRpcRequestMessage {
    Initialize(InitializeRequest),
    Ping(PingRequest),
    Complete(CompleteRequest),
    ListRoots(ListRootsRequest),
    ListPrompts(ListPromptsRequest),
    GetPrompt(GetPromptRequest),
    ListResources(ListResourcesRequest),
    ListResourceTemplates(ListResourceTemplatesRequest),
    ReadResource(ReadResourceRequest),
    Subscribe(SubscribeRequest),
    Unsubscribe(UnsubscribeRequest),
    CallTool(CallToolRequest),
    ListTools(ListToolsRequest),
    SetLevel(SetLevelRequest),
    CreateMessage(CreateMessageRequest),
}

/// A unified type for handling all JSON-RPC notifications.
#[derive(Message)]
#[rtype(result = "()")]
pub enum JsonRpcNotificationMessage {
    Initialized(InitializedNotification),
    Cancelled(CancelledNotification),
    Progress(ProgressNotification),
    RootsListChanged(RootsListChangedNotification),
    LoggingMessage(LoggingMessageNotification),
    ResourceListChanged(ResourceListChangedNotification),
    ResourceUpdated(ResourceUpdatedNotification),
    ToolListChanged(ToolListChangedNotification),
    PromptListChanged(PromptListChangedNotification),
}


/* ==== REQUESTS (Expecting a Response) ==== */

macro_rules! create_request {
    ($struct_name:ident, $method:expr) => {
        #[derive(Message, Debug, Clone, Serialize, Deserialize,)]
        #[rtype(result = "Result<JsonRpcResponse, ()>")]
        pub struct $struct_name {
            pub request: JsonRpcRequest,
        }

        impl $struct_name {
            pub const METHOD: &'static str = $method;
        }
    };
}

// Define all MCP standard requests
create_request!(InitializeRequest, "initialize");
create_request!(InitializedNotificationRequest, "notifications/initialized");
create_request!(PingRequest, "ping");
create_request!(CompleteRequest, "completion/complete");
create_request!(ListRootsRequest, "roots/list");
create_request!(ListPromptsRequest, "prompts/list");
create_request!(GetPromptRequest, "prompts/get");
create_request!(ListResourcesRequest, "resources/list");
create_request!(ListResourceTemplatesRequest, "resources/templates/list");
create_request!(ReadResourceRequest, "resources/read");
create_request!(SubscribeRequest, "resources/subscribe");
create_request!(UnsubscribeRequest, "resources/unsubscribe");
create_request!(CallToolRequest, "tools/call");
create_request!(ListToolsRequest, "tools/list");
create_request!(SetLevelRequest, "logging/setLevel");
create_request!(CreateMessageRequest, "sampling/createMessage");

/* ==== NOTIFICATIONS (Do Not Expect a Response) ==== */

macro_rules! create_notification {
    ($struct_name:ident, $method:expr) => {
        #[derive(Message, Debug, Clone, Serialize, Deserialize,)]
        #[rtype(result = "()")]
        pub struct $struct_name {
            pub params: Option<Value>,
        }

        impl $struct_name {
            pub const METHOD: &'static str = $method;
        }
    };
}

// Define all MCP standard notifications
create_notification!(InitializedNotification, "notifications/initialized");
create_notification!(CancelledNotification, "notifications/cancelled");
create_notification!(ProgressNotification, "notifications/progress");
create_notification!(RootsListChangedNotification, "notifications/roots/list_changed");
create_notification!(LoggingMessageNotification, "notifications/message");
create_notification!(ResourceListChangedNotification, "notifications/resources/list_changed");
create_notification!(ResourceUpdatedNotification, "notifications/resources/updated");
create_notification!(ToolListChangedNotification, "notifications/tools/list_changed");
create_notification!(PromptListChangedNotification, "notifications/prompts/list_changed");

/* ==== Message Type Enum for Handling All Requests/Notifications ==== */

/// Enum representing all MCP messages.
#[derive(Debug, Clone)]
pub enum McpMessage {
    Request(JsonRpcRequest),
    Response(JsonRpcResponse),
    Notification(String, Option<Value>), // (method, params)
}

impl McpMessage {
    /// Parses a JSON-RPC message into a structured `McpMessage`.
    pub fn from_json(value: Value) -> Result<Self, String> {
        if let Some(jsonrpc) = value.get("jsonrpc") {
            if jsonrpc != JSONRPC_VERSION {
                return Err("Invalid JSON-RPC version".to_string());
            }
        }

        if let Some(method) = value.get("method").and_then(|v| v.as_str()) {
            if value.get("id").is_some() {
                // It's a request
                serde_json::from_value::<JsonRpcRequest>(value.clone())
                    .map(McpMessage::Request)
                    .map_err(|e| e.to_string())
            } else {
                // It's a notification
                let params = value.get("params").cloned();
                Ok(McpMessage::Notification(method.to_string(), params))
            }
        } else if value.get("id").is_some() {
            // It's a response
            serde_json::from_value::<JsonRpcResponse>(value.clone())
                .map(McpMessage::Response)
                .map_err(|e| e.to_string())
        } else {
            Err("Unknown MCP message type".to_string())
        }
    }
}
