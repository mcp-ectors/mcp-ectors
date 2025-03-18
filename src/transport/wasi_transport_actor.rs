use actix::prelude::*;
use mcp_spec::protocol::{ErrorData, JsonRpcError, JsonRpcResponse};

use crate::{client::ClientRegistryActor, mcp::{InitializationActor, ListPromptsActor, ListResourcesActor, ListToolsActor}, messages::transport_messages::{StartTransport, StopTransport, TransportRequest, TransportResponse}, router::router_registry::ActorRouterRegistry};
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
    engine: Engine,
    _store: Store<()>,
    module: Module,
    _registry_addr: Addr<ClientRegistryActor>,
    _router_registry: ActorRouterRegistry,
    _prompts: Addr<ListPromptsActor>,
    _tools: Addr<ListToolsActor>,
    _resources: Addr<ListResourcesActor>,
    
}

impl WasiTransportActor
{
    pub fn new(
        config: WasiTransportConfig,
        _registry_addr: Addr<ClientRegistryActor>,
        _router_registry: ActorRouterRegistry,
        _prompts: Addr<ListPromptsActor>,
        _tools: Addr<ListToolsActor>,
        _resources: Addr<ListResourcesActor>,
    ) -> Result<Self, String> {
        let engine = Engine::default();
        let _store = Store::new(&engine, ());
        let module = Module::from_file(&engine, config.wasm_path)
            .map_err(|e| format!("Failed to load WASM module: {:?}", e))?;

        Ok(Self {
            engine,
            _store,
            module,
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
        router_registry: ActorRouterRegistry,
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
    type Result = ResponseActFuture<Self, Result<JsonRpcResponse, JsonRpcError>>;

    fn handle(&mut self, msg: TransportRequest, _ctx: &mut Self::Context) -> Self::Result {
        let request = msg.request.clone();
        let engine = self.engine.clone();
        let module = self.module.clone();

        let fut = async move {
            let mut store = Store::new(&engine, ());
            let _linker: Linker<()> = Linker::new(&engine);
            let instance = Instance::new(&mut store, &module, &[]);

            let result = match instance {
                Ok(instance) => {
                    let function = instance.get_typed_func::<(), ()>(&mut store, "handle_request");
                    match function {
                        Ok(func) => {
                            func.call(&mut store, ()).ok();
                            JsonRpcResponse {
                                jsonrpc: "2.0".to_string(),
                                id: request.id,
                                result: Some(serde_json::json!({ "message": "Processed by WASI module" })),
                                error: None,
                            }
                        }
                        Err(_) => JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            id: request.id,
                            result: None,
                            error: Some(ErrorData {
                                code: -32000,
                                message: "Failed to call WASI function".to_string(),
                                data: None,
                            }),
                        },
                    }
                }
                Err(_) => JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(ErrorData {
                        code: -32000,
                        message: "Failed to instantiate WASI module".to_string(),
                        data: None,
                    }),
                },
            };

            Ok(result)
        };

        Box::pin(actix::fut::wrap_future(fut))
    }
}

/// Handles sending responses
impl Handler<TransportResponse> for WasiTransportActor 
{
    type Result = ();

    fn handle(&mut self, msg: TransportResponse, _ctx: &mut Self::Context) -> Self::Result {
        info!("WasiTransportActor response: {:?}", msg.response);
    }
}
