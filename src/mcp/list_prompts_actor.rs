use actix::prelude::*;
use crate::{messages::{AddPromptsRequest, ListPromptsRequest, RemovePromptsRequest}, router::router_registry::ROUTER_SEPERATOR};
use mcp_spec::{prompt::Prompt, protocol::{JsonRpcResponse, ListPromptsResult}};


/// **A simple prompt router that implements `RouterHandler`**
#[derive(Clone)]
pub struct ListPromptsActor {
    prompts: Vec<Prompt>,
}

impl ListPromptsActor {
    pub fn new() -> Self {
        Self {
            prompts:Vec::new(),
        }
    }

    fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts.clone()
    }

    pub fn add_prompts(&mut self, new_prompts: Vec<Prompt>) {
        self.prompts.extend(new_prompts);
    }

    fn remove_prompts(&mut self, prompts_to_remove: Vec<Prompt>){
        for prompt in prompts_to_remove {
            self.prompts.retain(|existing_prompt| existing_prompt != &prompt);
        }
    }
}


/// **Actix Actor implementation for PromptRouter**
impl Actor for ListPromptsActor {
    type Context = Context<Self>;
}

/// **Actix Handler for `RouterRequest`**
impl Handler<ListPromptsRequest> for ListPromptsActor {
    type Result = ResponseFuture<Result<JsonRpcResponse, ()>>;

    fn handle(&mut self, msg: ListPromptsRequest, _ctx: &mut Self::Context) -> Self::Result {
        let request = msg.request.clone();
        let prompts = self.list_prompts();
        let fut = async move {
            let result = ListPromptsResult{ prompts };
             // Acquire the read lock
            Ok(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id.clone(),
                result: Some(serde_json::json!(result.clone())), // Clone the prompts list to avoid holding the lock
                error: None,
            })
        };
        Box::pin(fut)
    }
}
impl<T> Handler<AddPromptsRequest<T>> for ListPromptsActor
where
    T: Actor<Context = Context<T>> + Unpin + Send + 'static,
{
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: AddPromptsRequest<T>, _ctx: &mut Self::Context) -> Self::Result {
        // Create a new vector of prompts with the router_id:name substitution
        let new_prompts: Vec<Prompt> = msg
            .prompts
            .into_iter()
            .enumerate()
            .map(|(_i, prompt)| {
                // Substitute router_id:name into the name of each prompt
                let new_name = format!("{}{}{}", msg.router_id, ROUTER_SEPERATOR, prompt.name);
                
                // Create new Prompt with updated name and keep description and arguments intact
                Prompt {
                    name: new_name,
                    description: prompt.description.clone(),
                    arguments: prompt.arguments.clone(),
                }
            })
            .collect();
        // Call self.add_prompts to add the new prompts to the actor
        self.add_prompts(new_prompts);

        // Optionally, send the result back to the router if needed
        Box::pin(async { Ok(()) }) // Return a completed future with a success result
    }
}

impl<T> Handler<RemovePromptsRequest<T>> for ListPromptsActor
where
    T: Actor<Context = Context<T>> + Unpin + Send + 'static,
{
    type Result = ResponseFuture<Result<(), ()>>;

    fn handle(&mut self, msg: RemovePromptsRequest<T>, _ctx: &mut Self::Context) -> Self::Result {
        // Create a new vector of prompts with the router_id:name substitution
        let old_prompts: Vec<Prompt> = msg
            .prompts
            .into_iter()
            .enumerate()
            .map(|(_i, prompt)| {
                // Substitute router_id:name into the name of each prompt
                let new_name = format!("{}{}{}", msg.router_id, ROUTER_SEPERATOR, prompt.name);
                
                // Create new Prompt with updated name and keep description and arguments intact
                Prompt {
                    name: new_name,
                    description: prompt.description.clone(),
                    arguments: prompt.arguments.clone(),
                }
            })
            .collect();
        // Call self.add_prompts to add the new prompts to the actor
        self.remove_prompts(old_prompts);

        // Optionally, send the result back to the router if needed
        Box::pin(async { Ok(()) }) // Return a completed future with a success result
    }
}