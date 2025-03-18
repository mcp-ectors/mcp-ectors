use actix::prelude::*;
use crate::{messages::{AddResourcesRequest, ListResourcesRequest, RemoveResourcesRequest}, router::router_registry::ROUTER_SEPERATOR};
use mcp_spec::{protocol::{JsonRpcResponse, ListResourcesResult}, resource::Resource};


/// **A simple resource router that implements `RouterHandler`**
#[derive(Clone)]
pub struct ListResourcesActor {
    resources: Vec<Resource>,
}

impl ListResourcesActor {
    pub fn new() -> Self {
        Self {
            resources:Vec::new(),
        }
    }

    fn list_resources(&self) -> Vec<Resource> {
        self.resources.clone()
    }

    pub fn add_resources(&mut self, new_resources: Vec<Resource>) {
        self.resources.extend(new_resources);
    }

    fn remove_resources(&mut self, resources_to_remove: Vec<Resource>){
        for resource in resources_to_remove {
            self.resources.retain(|existing_resource| existing_resource != &resource);
        }
    }
}


/// **Actix Actor implementation for ResourceRouter**
impl Actor for ListResourcesActor {
    type Context = Context<Self>;
}

/// **Actix Handler for `RouterRequest`**
impl Handler<ListResourcesRequest> for ListResourcesActor {
    type Result = ResponseFuture<Result<JsonRpcResponse, ()>>;

    fn handle(&mut self, msg: ListResourcesRequest, _ctx: &mut Self::Context) -> Self::Result {
        let request = msg.request.clone();
        let resources = self.list_resources();
        let fut = async move {
            let result = ListResourcesResult{ resources, next_cursor: None };
             // Acquire the read lock
            Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(serde_json::json!(result.clone())), // Clone the resources list to avoid holding the lock
                error: None,
            })
        };
        Box::pin(fut)
    }
}
impl<T> Handler<AddResourcesRequest<T>> for ListResourcesActor
where
    T: Actor<Context = Context<T>> + Unpin + Send + 'static,
{
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: AddResourcesRequest<T>, _ctx: &mut Self::Context) -> Self::Result {
        // Create a new vector of resources with the router_id:name substitution
        let new_resources: Vec<Resource> = msg
            .resources
            .into_iter()
            .enumerate()
            .map(|(_i, resource)| {
                // Substitute router_id:name into the name of each resource
                let new_name = format!("{}{}{}", msg.router_id, ROUTER_SEPERATOR, resource.name);
                
                // Create new Resource with updated name and keep description and arguments intact
                Resource {
                    name: new_name,
                    description: resource.description.clone(),
                    uri: resource.uri.clone(),
                    mime_type: resource.mime_type.clone(),
                    annotations: resource.annotations.clone(),
                }
            })
            .collect();
        // Call self.add_resources to add the new resources to the actor
        self.add_resources(new_resources);

        // Optionally, send the result back to the router if needed
        Box::pin(async { Ok(()) }) // Return a completed future with a success result
    }
}

impl<T> Handler<RemoveResourcesRequest<T>> for ListResourcesActor
where
    T: Actor<Context = Context<T>> + Unpin + Send + 'static,
{
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: RemoveResourcesRequest<T>, _ctx: &mut Self::Context) -> Self::Result {
        // Create a new vector of resources with the router_id:name substitution
        let old_resources: Vec<Resource> = msg
            .resources
            .into_iter()
            .enumerate()
            .map(|(_i, resource)| {
                // Substitute router_id:name into the name of each resource
                let new_uri = format!("{}{}{}", msg.router_id, ROUTER_SEPERATOR, resource.uri);
                
                // Create new Resource with updated name and keep description and arguments intact
                Resource {
                    name: resource.name.clone(),
                    description: resource.description.clone(),
                    uri: new_uri,
                    mime_type: resource.mime_type.clone(),
                    annotations: resource.annotations.clone(),
                }
            })
            .collect();
        // Call self.add_resources to add the new resources to the actor
        self.remove_resources(old_resources);

        // Optionally, send the result back to the router if needed
        Box::pin(async { Ok(()) }) // Return a completed future with a success result
    }
}