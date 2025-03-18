use actix::prelude::*;
use crate::{messages::{AddToolsRequest, ListToolsRequest, RemoveToolsRequest}, router::router_registry::ROUTER_SEPERATOR};
use mcp_spec::{protocol::{JsonRpcResponse, ListToolsResult}, tool::Tool};


/// **A simple tool router that implements `RouterHandler`**
#[derive(Clone)]
pub struct ListToolsActor {
    tools: Vec<Tool>,
}

impl ListToolsActor {
    pub fn new() -> Self {
        Self {
            tools:Vec::new(),
        }
    }

    fn list_tools(&self) -> Vec<Tool> {
        self.tools.clone()
    }

    pub fn add_tools(&mut self, new_tools: Vec<Tool>) {
        self.tools.extend(new_tools);
    }

    fn remove_tools(&mut self, tools_to_remove: Vec<Tool>){
        for tool in tools_to_remove {
            self.tools.retain(|existing_tool| existing_tool != &tool);
        }
    }
}


/// **Actix Actor implementation for ToolRouter**
impl Actor for ListToolsActor {
    type Context = Context<Self>;
}

/// **Actix Handler for `RouterRequest`**
impl Handler<ListToolsRequest> for ListToolsActor {
    type Result = ResponseFuture<Result<JsonRpcResponse, ()>>;

    fn handle(&mut self, msg: ListToolsRequest, _ctx: &mut Self::Context) -> Self::Result {
        let request = msg.request.clone();
        let tools = self.list_tools();
        let fut = async move {
            let result = ListToolsResult{ tools, next_cursor: None };
             // Acquire the read lock
            Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(serde_json::json!(result.clone())), // Clone the tools list to avoid holding the lock
                error: None,
            })
        };
        Box::pin(fut)
    }
}
impl<T> Handler<AddToolsRequest<T>> for ListToolsActor
where
    T: Actor<Context = Context<T>> + Unpin + Send + 'static,
{
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: AddToolsRequest<T>, _ctx: &mut Self::Context) -> Self::Result {
        // Create a new vector of tools with the router_id:name substitution
        let new_tools: Vec<Tool> = msg
            .tools
            .into_iter()
            .enumerate()
            .map(|(_i, tool)| {
                // Substitute router_id:name into the name of each tool
                let new_name = format!("{}{}{}", msg.router_id, ROUTER_SEPERATOR, tool.name);
                
                // Create new Tool with updated name and keep description and arguments intact
                Tool {
                    name: new_name,
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                }
            })
            .collect();
        // Call self.add_tools to add the new tools to the actor
        self.add_tools(new_tools);

        // Optionally, send the result back to the router if needed
        Box::pin(async { Ok(()) }) // Return a completed future with a success result
    }
}

impl<T> Handler<RemoveToolsRequest<T>> for ListToolsActor
where
    T: Actor<Context = Context<T>> + Unpin + Send + 'static,
{
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: RemoveToolsRequest<T>, _ctx: &mut Self::Context) -> Self::Result {
        // Create a new vector of tools with the router_id:name substitution
        let old_tools: Vec<Tool> = msg
            .tools
            .into_iter()
            .enumerate()
            .map(|(_i, tool)| {
                // Substitute router_id:name into the name of each tool
                let new_name = format!("{}{}{}", msg.router_id, ROUTER_SEPERATOR, tool.name);
                
                // Create new Tool with updated name and keep description and arguments intact
                Tool {
                    name: new_name,
                    description: tool.description.clone(),
                    input_schema: tool.input_schema.clone(),
                }
            })
            .collect();
        // Call self.add_tools to add the new tools to the actor
        self.remove_tools(old_tools);

        // Optionally, send the result back to the router if needed
        Box::pin(async { Ok(()) }) // Return a completed future with a success result
    }
}