
use actix::prelude::*;
use mcp_spec::protocol::JsonRpcRequest;
use mcp_spec::protocol::JsonRpcResponse;
use mcp_spec::protocol::JsonRpcError;
use crate::transport::TransportError;


/// Message sent from TransportManager to a Transport Actor to start processing
#[derive(Message)]
#[rtype(result = "Result<(), TransportError>")]
pub struct StartTransport;

/// Message sent from a transport to the RouterRegistry to handle a request
#[derive(Message)]
#[rtype(result = "Result<JsonRpcResponse, JsonRpcError>")]
pub struct TransportRequest {
    pub request: JsonRpcRequest,
}

/// Message to stop a transport (graceful shutdown)
#[derive(Message)]
#[rtype(result = "()")]
pub struct StopTransport;
