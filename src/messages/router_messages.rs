use std::marker::PhantomData;
use actix::prelude::*;
use mcp_spec::{prompt::Prompt, protocol::{JsonRpcMessage, JsonRpcRequest,JsonRpcResponse, ServerCapabilities}, Resource, Tool};
use serde_json::Value;

use crate::router::RouterActor;
/// **Message to handle JSON-RPC requests**
#[derive(Message)]
#[rtype(result = "Result<JsonRpcResponse, ()>")]
pub struct HandleRequestMsg {
    pub request: JsonRpcRequest,
}

/// **Message to handle notifications**
#[derive(Message)]
#[rtype(result = "Result<Value, ()>")]
pub struct HandleNotificationMsg {
    pub topic: String,
    pub payload: Value,
}


/// Message sent from TransportManager to RouterRegistry to find the right router.
#[derive(Message)]
#[rtype(result = "Option<Addr<RouterActor>>")]
pub struct GetRouter {
    pub router_id: String,
    pub(crate) _marker: PhantomData<RouterActor>, // ✅ This makes Rust happy.
}

/// Message to register a new router (Native or WASM) along with its capabilities.
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct RegisterRouter {
    pub router_id: String,
    pub router_addr: Addr<RouterActor>,
    pub capabilities: Option<ServerCapabilities>,
}


/// Message to unregister a router (for dynamic removal).
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterRouter {
    pub router_id: String,
}

/// Message sent to a router to process an MCP request.
/*#[derive(Message)]
#[rtype(result = "Result<JsonRpcResponse, ()>")]
pub struct RouterRequest {
    pub router_id: String,
    pub request: JsonRpcRequest,
}
*/

/// Message from a router to TransportManager with the final response.
#[derive(Message)]
#[rtype(result = "()")]
pub struct RouterResponse {
    pub client_id: u64,
    pub response: JsonRpcMessage,
}

/// Message to retrieve the aggregated server capabilities.
#[derive(Message)]
#[rtype(result = "serde_json::Value")]
pub struct GetServerCapabilities;

/// Request to the list prompts actor to add prompts
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct AddPromptsRequest<T: Actor<Context = Context<T>> + Unpin + Send + 'static> {
    pub router_id: String,
    pub prompts: Vec<Prompt>,
    pub router: Addr<T>,
}

/// Request to the list prompts actor to remove prompts
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct RemovePromptsRequest<T: Actor<Context = Context<T>> + Unpin + Send + 'static> {
    pub router_id: String,
    pub prompts: Vec<Prompt>,
    pub router: Addr<T>,
}

/// Request to the list tools actor to add tools
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct AddToolsRequest<T: Actor<Context = Context<T>> + Unpin + Send + 'static> {
    pub router_id: String,
    pub tools: Vec<Tool>,
    pub router: Addr<T>,
}

/// Request to the list tools actor to remove prompts
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct RemoveToolsRequest<T: Actor<Context = Context<T>> + Unpin + Send + 'static> {
    pub router_id: String,
    pub tools: Vec<Tool>,
    pub router: Addr<T>,
}

/// Request to the list resources actor to add resources
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct AddResourcesRequest<T: Actor<Context = Context<T>> + Unpin + Send + 'static> {
    pub router_id: String,
    pub resources: Vec<Resource>,
    pub router: Addr<T>,
}

/// Request to the list resources actor to remove resources
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct RemoveResourcesRequest<T: Actor<Context = Context<T>> + Unpin + Send + 'static> {
    pub router_id: String,
    pub resources: Vec<Resource>,
    pub router: Addr<T>,
}