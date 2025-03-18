use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, MessageResult, ResponseFuture};
use mcp_spec::protocol::{JsonRpcResponse, ServerCapabilities};
use serde_json::json;
use tracing::error;

use crate::messages::{GetRouter, GetServerCapabilities, RegisterRouter, TransportRequest, UnregisterRouter};

use super::{Router, RouterActor};

// The WASI registry type
pub struct WasiRouterRegistry<R>
where
    R: Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    routers: HashMap<String, Addr<RouterActor<R>>>,
    capabilities: HashMap<String, ServerCapabilities>,
}

impl<R> WasiRouterRegistry<R>
where
    R:  Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }
}

impl<R> Actor for WasiRouterRegistry<R>
where
    R:  Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Context = Context<Self>;
}

impl<R> Handler<RegisterRouter<R>> for WasiRouterRegistry<R>
where
    R: Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Result = Result<(), ()>; 

    fn handle(&mut self, msg: RegisterRouter<R>, _ctx: &mut Self::Context) -> Self::Result {
        // Check if the router is already registered
        if self.routers.contains_key(&msg.router_id) {
            tracing::warn!("Router {} is already registered.", msg.router_id);
            return Err(());
        }

        tracing::info!("Registering router: {}", msg.router_id);
        self.routers.insert(msg.router_id.clone(), msg.router_addr);

        // If capabilities are provided, store them
        if let Some(capabilities) = msg.capabilities {
            self.capabilities.insert(msg.router_id, capabilities);
        }
        Ok(())
    }
}

impl<R> Handler<GetRouter<R>> for WasiRouterRegistry<R>
where
    R:Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Result = Option<Addr<RouterActor<R>>>;

    fn handle(&mut self, msg: GetRouter<R>, _ctx: &mut Self::Context) -> Self::Result {
        // Look up the router in the registry and return it
        self.routers.get(&msg.router_id).cloned() // Return the Addr of the router if found
    }
}

impl<R> Handler<UnregisterRouter> for WasiRouterRegistry<R>
where
    R: Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: UnregisterRouter, _ctx: &mut Self::Context) -> Self::Result {
        // Remove the router from the registry and its capabilities
        self.routers.remove(&msg.router_id);
        self.capabilities.remove(&msg.router_id);
    }
}

impl<R> Handler<GetServerCapabilities> for WasiRouterRegistry<R>
where
    R:  Router + Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Result = MessageResult<GetServerCapabilities>;

    fn handle(&mut self, _: GetServerCapabilities, _: &mut Self::Context) -> Self::Result {
        // Aggregate capabilities from the native registry
        let aggregated = json!({
            "capabilities": self.capabilities.keys().cloned().collect::<Vec<String>>(),
        });
        MessageResult(aggregated)  // Return wrapped in MessageResult
    }
}


impl<R> Handler<TransportRequest> for WasiRouterRegistry<R>
where
    R: Actor<Context = Context<R>> + Router + Send + Sync + Unpin + 'static,
{
    type Result= ResponseFuture<Result<JsonRpcResponse, ()>>;

    fn handle(&mut self, msg: TransportRequest, _ctx: &mut Self::Context) -> Self::Result {
       let method = msg.request.method.clone();
       let router = self.routers.get(&method).cloned();
       Box::pin(async move {
            match router {
                Some(router) => {
                    // Attempt to send the request to the router and flatten the result
                    match router.send(msg).await {
                        Ok(response) => response, // Return the response directly
                        Err(_) => {
                            // Log the error if sending the message failed
                            error!("Failed to send message to router for method: {}", method);
                            Err(()) // Return an error if message sending failed
                        }
                    }
                },
                None => {
                    // Log the error when no router is found for the method
                    error!("No router found for method: {}", method);
                    Err(()) // Return an error if no router is found
                }
            }
        })
    }
}
