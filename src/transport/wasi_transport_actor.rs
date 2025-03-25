use actix::prelude::*;
use mcp_spec::protocol::{JsonRpcError, JsonRpcResponse};

use crate::{client::ClientRegistryActor, mcp::{InitializationActor, ListPromptsActor, ListResourcesActor, ListToolsActor}, messages::transport_messages::{StartTransport, StopTransport, TransportRequest}, router::router_registry::ActorRouterRegistry, utils::{json_rpc::MCP_INVALID_REQUEST, JsonRpcUtils}};
use wasmtime::*;
use tracing::info;

use super::{TransportActorTrait, TransportError};
#[derive(Clone,Debug)]
pub struct WasiTransportConfig {
    pub wasm_path: String,
}

/// Actor for handling WASI-based JSON-RPC transport
pub struct WasiTransportActor
{
    _registry_addr: Addr<ClientRegistryActor>,
    _router_registry: Addr<ActorRouterRegistry>,
    _prompts: Addr<ListPromptsActor>,
    _tools: Addr<ListToolsActor>,
    _resources: Addr<ListResourcesActor>,
    
}

impl WasiTransportActor
{
    pub fn new(
       _config: WasiTransportConfig,
        _registry_addr: Addr<ClientRegistryActor>,
        _router_registry: Addr<ActorRouterRegistry>,
        _prompts: Addr<ListPromptsActor>,
        _tools: Addr<ListToolsActor>,
        _resources: Addr<ListResourcesActor>,
    ) -> Result<Self, String> {
        Ok(Self {

            _registry_addr,
            _router_registry,
            _prompts,
            _tools,
            _resources
        })
    }
}

impl TransportActorTrait for WasiTransportActor
{
    type Config = WasiTransportConfig;

    fn new(
        config: Self::Config,
        client_registry: Addr<ClientRegistryActor>,
        router_registry: Addr<ActorRouterRegistry>,
        _initialize: InitializationActor,
        prompts: Addr<ListPromptsActor>,
        tools: Addr<ListToolsActor>,
        resources: Addr<ListResourcesActor>,
    ) -> Self {
        WasiTransportActor::new(config, client_registry, router_registry, prompts, tools, resources).unwrap()
    }
}

impl Actor for WasiTransportActor 
{
    type Context = Context<Self>;
}

/// Handles starting the transport (initializing WASI)
impl Handler<StartTransport> for WasiTransportActor
{
    type Result = ResponseActFuture<Self, Result<(), TransportError>>;

    fn handle(&mut self, _msg: StartTransport, _ctx: &mut Self::Context) -> Self::Result {
        info!("WasiTransportActor started.");
        Box::pin(actix::fut::ready(Ok(())))
    }
}

/// Handles stopping the transport
impl Handler<StopTransport> for WasiTransportActor
{
    type Result = ();

    fn handle(&mut self, _msg: StopTransport, _ctx: &mut Self::Context) -> Self::Result {
        info!("WasiTransportActor stopping.");
    }
}

/// Handles incoming transport requests (executing WASI modules)
impl Handler<TransportRequest> for WasiTransportActor
{
    type Result = Result<JsonRpcResponse, JsonRpcError>;

    fn handle(&mut self, msg: TransportRequest, _ctx: &mut Self::Context) -> Self::Result {
        Err(JsonRpcUtils::error_response(msg.request.id, 
            MCP_INVALID_REQUEST, 
            format!("Did not expect this request: {:?}",msg.request).as_str(), 
        None))
    }
}

