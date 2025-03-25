use actix::{Actor, Addr, Context, Handler};

use crate::{client::ClientRegistryActor, mcp::{InitializationActor, ListPromptsActor, ListResourcesActor, ListToolsActor}, messages::{StartTransport, StopTransport, TransportRequest}, router::router_registry::ActorRouterRegistry};

pub trait TransportActorTrait
where
    Self: Actor<Context = Context<Self>> 
        + Handler<TransportRequest>
        + Handler<StartTransport>
        + Handler<StopTransport>,
    Self: Send + Sync + 'static,  // Ensure the actor itself is thread-safe and has a 'static lifetime
{
    type Config;
    
    /// **Create a new instance from the given configuration and dependencies**
    fn new(
        config: Self::Config,
        client_registry: Addr<ClientRegistryActor>,
        router_registry: Addr<ActorRouterRegistry>, // 🔹 Now works with any router registry that implements `RouterRegistry`
        initialize: InitializationActor,
        prompts: Addr<ListPromptsActor>,
        tools: Addr<ListToolsActor>,
        resources: Addr<ListResourcesActor>,
    ) -> Self;
}