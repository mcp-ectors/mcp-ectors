use actix::prelude::*;
use actix_web::dev::ServerHandle;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer, HttpResponse};
use actix_web::web::Data;
use actix_web_lab::sse::{Sse, Data as SseData, Event};
use futures::StreamExt;
use mcp_spec::protocol::{ErrorData, JsonRpcError, JsonRpcMessage, JsonRpcRequest, JsonRpcResponse};
use serde_json::Value;
use tracing::{error, info, trace};
use crate::client::ClientRegistryActor;
use crate::client::client_registry::{RegisterClient, NotifyClient}; 

use crate::mcp::{InitializationActor, ListPromptsActor, ListResourcesActor, ListToolsActor};
// Ensure these are imported correctly
use crate::messages::transport_messages::{TransportRequest, StartTransport, StopTransport};
use crate::messages::{BroadcastSseMessage, CallToolRequest, ClientMessage, DeregisterSseClient, GetPromptRequest, GetRouter, InitializeRequest, InitializedNotificationRequest, ListPromptsRequest, ListResourceTemplatesRequest, ListResourcesRequest, ListToolsRequest, NotifySseClient, ReadResourceRequest, RegisterSseClient, SubscribeRequest, UnsubscribeRequest, JSONRPC_VERSION};
use crate::router::router_registry::ActorRouterRegistry;
use crate::utils::json_rpc::{JSON_RPC_INTERNAL_ERROR, MCP_INTERNAL_SERVER_ERROR, MCP_INVALID_METHOD, MCP_INVALID_REQUEST, MCP_SERVICE_UNAVAILABLE};
use crate::utils::JsonRpcUtils;

use std::collections::HashMap;

use tokio::sync::mpsc;
use std::time::Duration;
use actix_web::error::Error;

use super::transport_actor::TransportActorTrait;
use super::TransportError; 



// Configuration struct for the server
#[derive(Clone,Debug)]
pub struct SseTransportConfig {
    pub port: u16,
    pub tls_cert: Option<String>,
    pub tls_key: Option<String>,
    pub log_dir: String,
    pub log_file: String,
}

// SseTransportActor to manage SSE client connectionspub struct SseTransportActor<R>
pub struct SseTransportActor
{
    clients: HashMap<u64, Recipient<ClientMessage>>, // Track connected SSE clients
    config: SseTransportConfig,
    registry_addr: Addr<ClientRegistryActor>,
    router_registry: Addr<ActorRouterRegistry>,
    initialize: InitializationActor,
    prompts: Addr<ListPromptsActor>,
    tools: Addr<ListToolsActor>,
    resources: Addr<ListResourcesActor>,
    server: Option<ServerHandle>,
}
impl SseTransportActor
{
    pub fn new(config: SseTransportConfig, 
        registry_addr: Addr<ClientRegistryActor>, 
        router_registry: Addr<ActorRouterRegistry>,
        initialize: InitializationActor,
        prompts: Addr<ListPromptsActor>,
        tools: Addr<ListToolsActor>,
        resources: Addr<ListResourcesActor>,
    ) -> Self {
        Self {
            clients: HashMap::new(),
            config,
            registry_addr,
            router_registry,
            initialize,
            prompts,
            tools,
            resources,
            server: None,
        }
    }
    
}

impl TransportActorTrait for SseTransportActor 
{
    type Config = SseTransportConfig;

    fn new(config: Self::Config,
           client_registry: Addr<ClientRegistryActor>,
           router_registry: Addr<ActorRouterRegistry>,
           initialize: InitializationActor,
           prompts: Addr<ListPromptsActor>,
           tools: Addr<ListToolsActor>,
           resources: Addr<ListResourcesActor>,
        )
           -> Self {
        SseTransportActor::new(config, client_registry, router_registry, initialize, prompts, tools,resources)
    }
}


impl Actor for SseTransportActor 
{
    type Context = Context<Self>;

}

/// Registers a new SSE client.
impl Handler<RegisterSseClient> for SseTransportActor 
{
    type Result = u64;

    fn handle(&mut self, msg: RegisterSseClient, _ctx: &mut Self::Context) -> Self::Result {
        let client_id = rand::random::<u64>(); // Generate unique client ID
        self.clients.insert(client_id, msg.recipient);
        tracing::info!("Registered SSE client with ID: {}", client_id);
        client_id
    }
}

/// Deregisters an SSE client when they disconnect.
impl Handler<DeregisterSseClient> for SseTransportActor
{
    type Result = ();

    fn handle(&mut self, msg: DeregisterSseClient, _ctx: &mut Self::Context) -> Self::Result {
        if self.clients.remove(&msg.client_id).is_some() {
            tracing::info!("Deregistered SSE client with ID: {}", msg.client_id);
        } else {
            tracing::warn!("Tried to deregister unknown SSE client: {}", msg.client_id);
        }
    }
}

/// Sends a message to a specific SSE client.
impl Handler<NotifySseClient> for SseTransportActor
{
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: NotifySseClient, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(recipient) = self.clients.get(&msg.client_id) {
            let fut = recipient.send(ClientMessage(msg.message.clone()));
            tracing::info!("Sending message to SSE client: {}", msg.client_id);
            Box::pin(actix::fut::wrap_future(async move {
                fut.await.map_err(|_| ())
            }))
        } else {
            tracing::warn!("SSE client {} not found", msg.client_id);
            Box::pin(actix::fut::ready(Err(())))
        }
    }
}

/// Broadcasts a message to all connected SSE clients.
impl Handler<BroadcastSseMessage> for SseTransportActor 
{
    type Result = ();

    fn handle(&mut self, msg: BroadcastSseMessage, _ctx: &mut Self::Context) -> Self::Result {
        tracing::info!("Broadcasting message to {} SSE clients", self.clients.len());
        for (_id, recipient) in self.clients.iter() {
            let _ = recipient.do_send(ClientMessage(msg.message.clone()));
        }
    }
}
impl Handler<TransportRequest> for SseTransportActor
{
    type Result = Result<JsonRpcResponse, JsonRpcError>;

    fn handle(&mut self, msg: TransportRequest, _ctx: &mut Self::Context) -> Self::Result {
        Err(JsonRpcUtils::error_response(msg.request.id, 
            MCP_INVALID_REQUEST, 
            format!("Did not expect this request: {:?}",msg.request).as_str(), 
        None))
    }
}


/// Handles sending SSE messages to the client.
impl Handler<ClientMessage> for SseRecipient {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _ctx: &mut Self::Context) {
        let data = serde_json::to_string(&msg.0).unwrap_or_else(|_| "{}".to_string());
        let event = Event::Data(SseData::new(data));
        if self.sender.try_send(event).is_err() {
            tracing::warn!("Failed to send SSE message to client");
        }
    }
}

/// Represents an SSE recipient that forwards messages to the client.
pub struct SseRecipient {
    sender: mpsc::Sender<Event>,
}

impl Actor for SseRecipient {
    type Context = Context<Self>;
}



/// Handles `StartTransport`
impl Handler<StartTransport> for SseTransportActor
{
    type Result = ResponseActFuture<Self, Result<(), TransportError>>; // Return Result<(), TransportError>
    fn handle(&mut self, _msg: StartTransport, _ctx: &mut Self::Context) -> Self::Result {
        
        tracing::info!("Starting SSE transport...");
        let addr_str = format!("0.0.0.0:{}", self.config.port);
        let registry_addr = self.registry_addr.clone();
        let router_registry = self.router_registry.clone();
        //let sse_transport_addr = ctx.address();
        let initialize = self.initialize.clone();
        let prompts = self.prompts.clone();
        let tools = self.tools.clone();
        let resources = self.resources.clone();

        // Wrap the async logic inside a future and ensure it resolves to `()`.

        // Attempt to bind the HTTP server.
        let server_result = HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .app_data(Data::new(registry_addr.clone()))
                .app_data(Data::new(router_registry.clone()))
                .app_data(Data::new(initialize.clone()))
                .app_data(Data::new(prompts.clone()))
                .app_data(Data::new(tools.clone()))
                .app_data(Data::new(resources.clone()))
                .route("/sse", web::get().to(sse_handler))
                .route("/messages/", web::post().to(post_handler))
        })
        .bind(addr_str.clone());

        let server = match server_result {
            Ok(srv) => srv,
            Err(e) => {
                error!("Could not start the server because of an error: {}",e);
                return Box::pin(actix::fut::ready(Err(
                    TransportError::ConfigurationError(format!(
                        "Failed to bind HTTP on {}: {:?}",
                        addr_str, e
                    )),
                )))
            }
        };

        let server = server.run();

        // Obtain a handle from the server so that we can stop it later.
        let handle = server.handle();
        // Store the handle in the actor.
        self.server = Some(handle);
        
        actix_web::rt::spawn(async move {
            if let Err(e) = server.await {
                tracing::error!("Server run error: {:?}", e);
            }
        });
            
    

        // Wrap the future inside a ResponseActFuture and return it.
        Box::pin(actix::fut::wrap_future(async { Ok(()) }))

    }
}



/// Handles `StopTransport`
impl Handler<StopTransport> for SseTransportActor 
{
    type Result = ();

    fn handle(&mut self, _msg: StopTransport, _ctx: &mut Self::Context) -> Self::Result {
        tracing::info!("Stopping SSE transport...");

        // Stop the server gracefully
        if let Some(handle) = self.server.take() {
            let _ = handle.stop(true);
            tracing::info!("SSE transport server stopped.");
        }
    }
}


// --- Helper functions for POST and SSE Handlers ---
async fn sse_handler(registry: Data<Addr<ClientRegistryActor>>) -> Sse<impl Stream<Item = Result<Event, Error>>> {
    let (tx, rx) = mpsc::channel::<Event>(10000);
    let sse_recipient = SseRecipient { sender: tx.clone() }.start();
    let client_id = registry
        .send(RegisterClient { recipient: sse_recipient.recipient() })
        .await
        .unwrap();

    let init_event: Event = SseData::new(format!("/messages/?session_id={}", client_id))
        .event("endpoint")
        .into();

    let stream = futures::stream::once(async { 
            Ok(init_event) 
        })
        .chain(futures::stream::unfold(rx, |mut rx| async {
            match rx.recv().await {
                Some(event) => {
                    tracing::info!("Received event: {:?}", event); // Add logging here
                    Some((Ok(event), rx))
                }
                None => {
                    tracing::warn!("Event stream closed");
                    None
                }
            }
        }));

    Sse::from_stream(stream).with_keep_alive(Duration::from_secs(15))
}

async fn post_handler(
    query: web::Query<HashMap<String, String>>,
    payload: web::Json<JsonRpcRequest>,
    registry: Data<Addr<ClientRegistryActor>>,
    router_registry: Data<Addr<ActorRouterRegistry>>,
    initialization_actor: Data<InitializationActor>,
    prompts: Data<Addr<ListPromptsActor>>,
    tools: Data<Addr<ListToolsActor>>,
    resources: Data<Addr<ListResourcesActor>>,
) -> Result<HttpResponse, Error>  
{
    let session_id = query.get("session_id")
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing session_id"))?;
    let client_id: u64 = session_id.parse()
        .map_err(|_| actix_web::error::ErrorBadRequest("Invalid session_id"))?;
    info!("Post request: {:?} from {}",payload.clone(),session_id);
    let response: Result<JsonRpcResponse, JsonRpcError> = match payload.method.as_str() {
        CallToolRequest::METHOD | GetPromptRequest::METHOD | ListResourceTemplatesRequest::METHOD => {
            trace!("Calling call tool/prompt");
            let id = payload.id.clone();
            let req = payload.into_inner();
            let att = "name".to_string();
            let params=  req.clone().params.expect("no params found");
            let action = params[att.clone()].as_str().expect("tool/prompt name not found");
            match router_request(id, action.to_string(), router_registry, req.clone(), att.clone()).await {
                Ok(res) => Ok(res),
                Err(err) => Err(err),
            }

        },
        ReadResourceRequest::METHOD | SubscribeRequest::METHOD | UnsubscribeRequest::METHOD => {
            tracing::trace!("Calling read/subscribe/unsubscribe resource");
            let id = payload.id.clone();
            let req = payload.into_inner();
            let att = "uri".to_string();
            let params=  req.clone().params.expect("no params found");
            let action = params[att.clone()].as_str().expect("resource uri not found");
            match router_request(id, action.to_string(), router_registry, req.clone(), att.clone()).await {
                Ok(res) => Ok(res),
                Err(err) => Err(err),
            }

        },
        InitializeRequest::METHOD => {
            // Handle InitializeRequest by calling InitializationActor
            tracing::info!("Received InitializeRequest");
            // Call the InitializationActor for InitializeRequest
            initialization_actor.handle_initialize_request(payload.0)
            
        },
        InitializedNotificationRequest::METHOD => {
            // Handle InitializedNotificationRequest by calling InitializationActor
            tracing::info!("Received InitializedNotificationRequest");
            // Call the InitializationActor for InitializedNotificationRequest
            initialization_actor.handle_initialized_notification_request(payload.0)

        },
        ListToolsRequest::METHOD => {
            tracing::trace!("Calling list tools");
            let id = payload.id.clone();
            let result = tools.send(ListToolsRequest{request: payload.into_inner()})
            .await
            .map_err(|e| JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_SERVICE_UNAVAILABLE, message: format!("Transport actor error: {}",e), data: None }, }).unwrap()
            .map_err(|e| JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_INTERNAL_SERVER_ERROR, message: format!("Processing actor error: {:?}",e), data: None }, }).unwrap();

            Ok(result)
        },
        ListPromptsRequest::METHOD => {
            tracing::trace!("Calling list prompts");
            let id = payload.id.clone();
            let result = prompts.send(ListPromptsRequest{request: payload.into_inner()})
            .await
            .map_err(|e| JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_SERVICE_UNAVAILABLE, message: format!("Transport actor error: {}",e), data: None }, }).unwrap()
            .map_err(|e| JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_INTERNAL_SERVER_ERROR, message: format!("Processing actor error: {:?}",e), data: None }, }).unwrap();

            Ok(result)

        },
        ListResourcesRequest::METHOD => {
            tracing::trace!("Calling list resources");
            let id = payload.id.clone();
            let result = resources.send(ListResourcesRequest{request: payload.into_inner()})
            .await
            .map_err(|e| JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_SERVICE_UNAVAILABLE, message: format!("Transport actor error: {}",e), data: None }, }).unwrap()
            .map_err(|e| JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_INTERNAL_SERVER_ERROR, message: format!("Processing actor error: {:?}",e), data: None }, }).unwrap();

            Ok(result)

        },
        method => {
            let id = payload.id.clone();
            Err(JsonRpcError{jsonrpc: JSONRPC_VERSION.to_owned(), id, error: ErrorData{code: MCP_INVALID_METHOD, message: format!("Invalid method: {}",method), data: None }, })
        }


    };

    match response {
        Ok(json_rpc_response) => {
            // Send the successful JsonRpcResponse to the client
            registry.do_send(NotifyClient {
                client_id,
                message: JsonRpcMessage::Response(json_rpc_response), // Pass the JsonRpcResponse directly
            });
        }
        Err(error) => {
            registry.do_send(NotifyClient {
                client_id,
                message: JsonRpcMessage::Error(error), // Pass the JsonRpcResponse directly
            });
        }
    };


    Ok(HttpResponse::Ok().json("Accepted"))
}

async fn router_request(id: Option<u64>, action: String, router_registry:Data<Addr<ActorRouterRegistry>>, req: JsonRpcRequest, attribute: String) -> Result<JsonRpcResponse,JsonRpcError> {
    let response = router_registry
        .send(GetRouter { router_id: action.clone(), _marker: std::marker::PhantomData })
        .await
        .unwrap();

    let (router, action) = match response {
        Some(response) => (Some(response.0),response.1),
        None => (None, action.clone()),
    };
    
    //let (router,action) = router_registry.get_router(action);
    // replace whatever parameter had the router_id:action with only action, e.g. hello_world_actor:hello
    let mut req_cloned = req.clone();
    // Check if `params` is `Some` and modify the attribute accordingly
    if let Some(ref mut params) = req_cloned.params {
        if let Some(param_value) = params.get_mut(attribute) {
            // Set the new value for `attribute`
            *param_value = Value::String(action.clone());
        }
    }

    match router {
        Some(router) => {
            match router.send(TransportRequest{request:req_cloned}).await {
                Ok(response) => match response {
                    Ok(json_rpc_response) => Ok(json_rpc_response),
                    Err(error) => {
                        error!("Failed to send {:?} to actor", action.clone());
                        Err(JsonRpcUtils::error_response(id, 
                        MCP_INVALID_REQUEST, 
                        format!("transport error: {:?}",error).as_str(), 
                        None))
                    }
                }, // Successfully retrieved response
                Err(_) => {
                    // Log error if sending the message failed
                    error!("Failed to send {:?} to router", action);
                    Err(JsonRpcUtils::error_response(id, 
                        JSON_RPC_INTERNAL_ERROR, 
                        "transport error: ", 
                        None))
                }
            }
        }
        None => {
            error!("Failed to find router for {:?}", req);
            Err(JsonRpcUtils::error_response(id, 
                JSON_RPC_INTERNAL_ERROR, 
                format!("transport error, no router for {}", action).as_str(), 
            None))

        }
    }
} 