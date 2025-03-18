use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler, MessageResult};

use serde_json::json;

use crate::messages::{GetRouter, GetServerCapabilities, RegisterRouter, UnregisterRouter};

use super::router_registry::{RouterCapabilities, RouterRegistry};


// The Actor-based registry type
pub struct ActorRouterRegistry<R>
where
    R: Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    routers: HashMap<String, Addr<R>>,
    capabilities: HashMap<String, RouterCapabilities>,
}

impl<R> ActorRouterRegistry<R>
where
    R:  Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    pub fn new() -> Self {
        Self {
            routers: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }
}

impl<R> Actor for ActorRouterRegistry<R>
where
    R:  Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Context = Context<Self>;
}

impl<R> RouterRegistry<R> for ActorRouterRegistry<R>
where
    R:  Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    fn register_router(&mut self, router_id: String, router_addr: Addr<R>) {
        self.routers.insert(router_id, router_addr);
    }

    fn get_router(&self, router_id: &str) -> Option<Addr<R>> {
        self.routers.get(router_id).cloned()
    }

    fn unregister_router(&mut self, router_id: &str) {
        self.routers.remove(router_id);
        self.capabilities.remove(router_id);
    }

    fn get_capabilities(&self) -> Vec<String> {
        self.capabilities.keys().cloned().collect()
    }
}


impl<R> Handler<RegisterRouter<R>> for ActorRouterRegistry<R>
where
    R:  Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
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

impl<R> Handler<GetRouter<R>> for ActorRouterRegistry<R>
where
    R:  Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Result = Option<Addr<R>>;

    fn handle(&mut self, msg: GetRouter<R>, _ctx: &mut Self::Context) -> Self::Result {
        // Look up the router in the registry and return it
        self.routers.get(&msg.router_id).cloned() // Return the Addr of the router if found
    }
}

impl<R> Handler<UnregisterRouter> for ActorRouterRegistry<R>
where
    R: Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
{
    type Result = ();

    fn handle(&mut self, msg: UnregisterRouter, _ctx: &mut Self::Context) -> Self::Result {
        // Remove the router from the registry and its capabilities
        self.routers.remove(&msg.router_id);
        self.capabilities.remove(&msg.router_id);
    }
}

impl<R> Handler<GetServerCapabilities> for ActorRouterRegistry<R>
where
    R: Actor<Context = Context<R>> + Send + Sync + Unpin + 'static,
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
