
use std::sync::Arc;

use actix::{Actor, Addr};
use tracing::info;

use crate::{mcp::{ListPromptsActor, ListResourcesActor, ListToolsActor}, messages::{AddPromptsRequest, AddResourcesRequest, AddToolsRequest}};

use super::{router_registry::{ActorRouterRegistry, RouterRegistry}, Router, RouterActor, SystemRouter};

pub enum RegistryType {
    Native,
    //Actor,
    Wasi,
}



pub struct RouterServiceManager
{
    list_prompts:Addr<ListPromptsActor>,
    list_tools:Addr<ListToolsActor>,
    list_resources:Addr<ListResourcesActor>,
    active_registry: ActorRouterRegistry,
}



impl RouterServiceManager
{
    fn new() -> Self {
        let active_registry = ActorRouterRegistry::new();
        let list_prompts= ListPromptsActor::new().start();
        let list_tools = ListToolsActor::new().start();
        let list_resources = ListResourcesActor::new().start();
        Self {
            list_prompts,
            list_tools,
            list_resources,
            active_registry,
        }
    }

    pub async fn default() -> Self {
        let mut manager = RouterServiceManager::new();
        let system = SystemRouter::new();
        let _ = manager
            .register_router::<SystemRouter>("system".to_string(), Box::new(system))
            .await;
        manager
    }

    pub async fn register_router<T: Router>(&mut self, router_id: String, router: Box<dyn Router>) -> Result<(),String> {
        let tools = router.list_tools();
        let resources = router.list_resources();
        let prompts = router.list_prompts();
        let _capabilities = &router.capabilities().clone();
        let router_addr = RouterActor::new(Arc::new(router)).start();
        info!("Registering router {} at {:?}", router_id.clone(), router_addr.clone());
        self.active_registry.register_router(router_id.clone(), router_addr.clone())?;

        self.list_prompts.do_send(AddPromptsRequest { router_id:router_id.clone(), prompts, router: router_addr.clone() });
        self.list_tools.do_send(AddToolsRequest { router_id:router_id.clone(), tools, router: router_addr.clone() });
        self.list_resources.do_send(AddResourcesRequest { router_id:router_id.clone(), resources, router: router_addr.clone() });
        

        Ok(())
    }



    pub async fn get_router(&self, action: String) -> (Option<Addr<RouterActor>>,String) {
       self.active_registry.get_router(action.clone())
    }


    pub fn get_registry(&self) -> ActorRouterRegistry {
        self.active_registry.clone()
    }

    pub fn get_list_prompts(&self) -> Addr<ListPromptsActor> {
        self.list_prompts.clone()
    }

    pub fn get_list_resources(&self) -> Addr<ListResourcesActor> {
        self.list_resources.clone()
    }

    pub fn get_list_tools(&self) -> Addr<ListToolsActor> {
        self.list_tools.clone()
    }

}
