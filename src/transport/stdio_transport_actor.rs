use actix::prelude::*;
use mcp_spec::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

use crate::{client::ClientRegistryActor, mcp::{InitializationActor, ListPromptsActor, ListResourcesActor, ListToolsActor}, messages::transport_messages::{StartTransport, StopTransport, TransportRequest}, router::router_registry::ActorRouterRegistry};
use std::io::{self, BufRead, Write};
use tokio::task;
use tracing::{info, error};

use super::{TransportActorTrait, TransportError};
#[derive(Clone,Debug)]
pub struct StdioTransportConfig;
/// Actor for handling Stdio (stdin/stdout) as a JSON-RPC transport
pub struct StdioTransportActor
{
    _router_registry: Addr<ActorRouterRegistry>,
}

impl Actor for StdioTransportActor
{
    type Context = Context<Self>;
}

impl StdioTransportActor
{
    pub fn new(router_registry: Addr<ActorRouterRegistry>) -> Self {
        Self { _router_registry: router_registry }
    }
}

impl TransportActorTrait for StdioTransportActor
{
    type Config = (); // No specific config needed

    fn new(
        _config: Self::Config,
        _client_registry: Addr<ClientRegistryActor>,
        router_registry: Addr<ActorRouterRegistry>,
        _initialize: InitializationActor,
        _prompts: Addr<ListPromptsActor>,
        _tools: Addr<ListToolsActor>,
        _resources: Addr<ListResourcesActor>,
    ) -> Self {
        Self::new(router_registry)
    }
}

/// Handles starting the transport (begin reading from stdin)
impl Handler<StartTransport> for StdioTransportActor
{
    type Result = ResponseActFuture<Self, Result<(), TransportError>>;

    fn handle(&mut self, _msg: StartTransport, ctx: &mut Self::Context) -> Self::Result {
        info!("StdioTransportActor started. Listening on stdin...");

        let addr = ctx.address();

        // Spawn a blocking task for stdin reading
        task::spawn_blocking(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            let mut buffer = String::new();

            while handle.read_line(&mut buffer).is_ok() {
                let request: Result<JsonRpcRequest, _> = serde_json::from_str(&buffer.trim());
                buffer.clear();

                match request {
                    Ok(req) => {
                        addr.do_send(TransportRequest { request: req });
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request from stdin: {:?}", e);
                    }
                }
            }
        });

        Box::pin(actix::fut::ready(Ok(())))
    }
}

/// Handles stopping the transport
impl Handler<StopTransport> for StdioTransportActor
{
    type Result = ();

    fn handle(&mut self, _msg: StopTransport, _ctx: &mut Self::Context) -> Self::Result {
        info!("StdioTransportActor stopping.");
    }
}

/// Handles incoming transport requests (process JSON-RPC)
impl Handler<TransportRequest> for StdioTransportActor
{
    type Result = ResponseActFuture<Self, Result<JsonRpcResponse, JsonRpcError>>;

    fn handle(&mut self, msg: TransportRequest, _ctx: &mut Self::Context) -> Self::Result {
        let request = msg.request.clone();
        info!("StdioTransportActor received request: {:?}", request);

        let fut = async move {
            let response = JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(serde_json::json!({ "message": "Processed by StdioTransportActor" })),
                error: None,
            };

            Ok(response)
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}


#[actix_rt::main]
async fn main() {
    let router_registry = ActorRouterRegistry::new().start();
    let transport_actor = StdioTransportActor::new(router_registry.clone()).start();

    // The `transport_actor` now processes requests from stdin.

    // Start a blocking task to handle stdin.
    task::spawn_blocking(move || {
        let stdin = io::stdin();
        let mut handle = stdin.lock();
        let mut buffer = String::new();

        loop {
            buffer.clear();
            // Read input from stdin
            if handle.read_line(&mut buffer).is_ok() {
                let request: Result<JsonRpcRequest, _> = serde_json::from_str(&buffer.trim());

                match request {
                    Ok(req) => {
                        // Send the request to the transport actor
                        let _reply = transport_actor.do_send(TransportRequest { request: req.clone() });

                        let id = req.id.clone();
                        // Simulate printing the response back to stdout
                        let response = JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: id,
                            result: Some(json!({"message": "Response from StdioTransportActor"})),
                            error: None,
                        };

                        // Print the response to stdout
                        let response_str = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
                        if let Err(e) = writeln!(io::stdout(), "{}", response_str) {
                            error!("Failed to write JSON-RPC response to stdout: {:?}", e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON-RPC request from stdin: {:?}", e);
                    }
                }
            } else {
                break; // Exit loop on failure to read
            }
        }
    });

    // This example runs indefinitely, handling stdin commands.
    // Implement a graceful shutdown if necessary.
    info!("StdioTransportActor is now listening for requests on stdin...");
}