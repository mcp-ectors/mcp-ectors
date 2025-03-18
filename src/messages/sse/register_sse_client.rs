use actix::prelude::*;

use crate::messages::ClientMessage;


/// Message to register a new SSE client
#[derive(Message)]
#[rtype(result = "u64")] // Returns a unique client ID
pub struct RegisterSseClient {
    pub recipient: Recipient<ClientMessage>,
}
