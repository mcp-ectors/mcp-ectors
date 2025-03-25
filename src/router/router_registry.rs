use std::collections::HashMap;

use actix::{Actor, Addr, Context, Handler};

use crate::messages::{GetRouter, RegisterRouter, UnregisterRouter};

use super::RouterActor;

pub const ROUTER_SEPERATOR: char = '_';

pub trait RouterRegistry {
    fn register_router(&mut self, router_id: String, router: Addr<RouterActor>) -> Result<(),String>;
    /// returns the address of the routeractor to call and the method to use to call
    fn get_router(&self, action: String) -> (Option<Addr<RouterActor>>,String);
    fn unregister_router(&mut self, router_id: &str);
}
#[derive(Clone)]
pub struct ActorRouterRegistry{
    routers: HashMap<String, Addr<RouterActor>>,
}

impl Actor for ActorRouterRegistry
{
    type Context = Context<Self>;
}

impl ActorRouterRegistry{
    pub fn new() -> Self{
        Self{
            routers: HashMap::new(),
        }
    }
}
fn split_at_seperator(input: String) -> (String, Option<String>) {
    if let Some(pos) = input.find(ROUTER_SEPERATOR) {
        let before = &input[..pos]; // Part before the colon
        let after = &input[pos + ROUTER_SEPERATOR.len_utf8()..]; // Part after the colon
        (before.to_string(), Some(after.to_string()))
    } else {
        (input, None) // If no colon is found, return the whole string as the first part and None for the second part
    }
}

// Implement RouterRegistry for HashMap
impl RouterRegistry for ActorRouterRegistry
{
    fn register_router(&mut self, router_id: String, router: Addr<RouterActor>) -> Result<(),String> {
        if router_id.contains("_") {
            return Err("A router id cannot conain an underscore [_]".to_string());
        }
        if self.routers.contains_key(&router_id){
            return Err(format!("A router id with id {} was already registered",&router_id));
        }
        self.routers.insert(router_id, router);
        Ok(())
    }

    fn get_router(&self, mut action: String) -> (Option<Addr<RouterActor>>,String) {
        // given we have many routers, we rewrite them as router_id:method, e.g. counter:call_tool
        let (router_id, action_opt) = split_at_seperator(action); 
        action = match action_opt {
            Some(action_ret) => action_ret,
            None => router_id.clone(),
        };
        (self.routers.get(router_id.as_str()).cloned(),action)
    }

    fn unregister_router(&mut self, router_id: &str) {
        self.routers.remove(router_id);
    }
}

impl Handler<GetRouter> for ActorRouterRegistry {
    type Result = Option<(Addr<RouterActor>,String)>;

    fn handle(&mut self, msg: GetRouter, _: &mut Self::Context) -> Self::Result {
        let (router_id, action_opt) = split_at_seperator(msg.router_id.clone());
        let action = action_opt.unwrap_or(router_id.clone());
        self.routers.get(&router_id)
            .cloned() // If router exists, clone and return it
            .map(|router| (router, action))
    }
}

impl Handler<RegisterRouter> for ActorRouterRegistry {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: RegisterRouter, _: &mut Self::Context) -> Self::Result {
        if msg.router_id.contains("_") {
            return Err(()); // Or handle the error appropriately
        }
        if self.routers.contains_key(&msg.router_id) {
            return Err(()); // Handle the error (router already exists)
        }

        // Register the router with the given ID
        self.routers.insert(msg.router_id, msg.router_addr);
        Ok(())
    }
}


impl Handler<UnregisterRouter> for ActorRouterRegistry {
    type Result = ();

    fn handle(&mut self, msg: UnregisterRouter, _: &mut Self::Context) {
        self.routers.remove(&msg.router_id);
    }
}

// A simple capability descriptor for a router
#[derive(Clone, Default)]
pub struct RouterCapabilities {
    pub tools_list_changed: bool,
    pub resources_subscribe: bool,
    pub resources_list_changed: bool,
    pub prompts_list_changed: bool,
}
