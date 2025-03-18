use actix::prelude::*;
use mcp_spec::protocol::JsonRpcMessage;



/// Message to notify a single SSE client
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct NotifySseClient {
    pub client_id: u64,
    pub message: JsonRpcMessage,
}
