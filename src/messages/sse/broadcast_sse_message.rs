use actix::prelude::*;
use mcp_spec::protocol::JsonRpcMessage;


/// Message to broadcast to all SSE clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastSseMessage {
    pub message: JsonRpcMessage,
}
