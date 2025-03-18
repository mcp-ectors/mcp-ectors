use actix::prelude::*;

/// Message to deregister an SSE client
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeregisterSseClient {
    pub client_id: u64,
}
