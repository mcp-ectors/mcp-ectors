use actix::prelude::*;
use mcp_spec::protocol::JsonRpcMessage;

/// Message to register a client in ClientRegistry
#[derive(Message)]
#[rtype(result = "u64")] // Returns a client ID
pub struct RegisterClient {
    pub recipient: Recipient<ClientMessage>,
}

/// Message to deregister a client from ClientRegistry
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeregisterClient {
    pub client_id: u64,
}

/// Message to send a JSON-RPC message to a specific client
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct NotifyClient {
    pub client_id: u64,
    pub message: JsonRpcMessage,
}

/// Message to broadcast a JSON-RPC message to all clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub message: JsonRpcMessage,
}

/// A message that clients will receive
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage(pub JsonRpcMessage);
