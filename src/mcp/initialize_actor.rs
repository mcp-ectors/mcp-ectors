use actix::prelude::*;
use mcp_spec::protocol::{ JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use serde_json::{json, Value};

use crate::messages::{ InitializedNotification, JSONRPC_VERSION};
use crate::router::router_registry::ROUTER_SEPERATOR;
use crate::router::topic_registry_actor::TopicMessage;
use crate::server_builder::{SERVER, VERSION};
#[derive(Clone)]
pub struct InitializationActor {
    pub server_capabilities: Value,
    pub protocol_version: String,
    pub server_info: (String, String), // (name, version)
    pub instructions: Option<String>,
}

impl InitializationActor {
    /// Constructs a new InitializationActor with default configuration.
    pub fn new() -> Self {
        Self {
            server_capabilities: json!({
                "logging": {},
                "prompts": { "listChanged": true },
                "resources": { "subscribe": true, "listChanged": true },
                "tools": { "listChanged": true }
            }),
            protocol_version: "2024-11-05".to_string(),
            server_info: (SERVER.to_string(), VERSION.to_string()),
            instructions: Some(format!("Please initialize your session. A multi mcp router server allows many routers to be installed. You can see this in the name of prompts and tools. They are formatted routerid{}prompt_name or routerid{}tool_name. The same for resource which are routerid{}uri. To get a list of what this server offers call resources/read with uri system{}all to understand which tools, prompts and resources this multi mcp router server has installed and what they do.",ROUTER_SEPERATOR,ROUTER_SEPERATOR,ROUTER_SEPERATOR,ROUTER_SEPERATOR)),
        }
    }

    pub fn handle_initialize_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse,JsonRpcError> {

        let protocol_version = self.protocol_version.clone();
        let server_capabilities = self.server_capabilities.clone();
        let server_info = self.server_info.clone();
        let instructions = self.instructions.clone();
        tracing::info!("Received Initialize from {:?}", req.id);

        let response = JsonRpcResponse {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id: req.id,
            result: Some(json!({
                "protocolVersion": protocol_version,
                "capabilities": server_capabilities,
                "serverInfo": {
                    "name": server_info.0,
                    "version": server_info.1,
                },
                "instructions": instructions,
            })),
            error: None,
        };

        Ok(response)

    }

    pub fn handle_initialized_notification_request(&self, req: JsonRpcRequest) -> Result<JsonRpcResponse,JsonRpcError> {
        tracing::info!("Received InitializedNotification");

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id,
            result: None,
            error: None,
        };

        Ok(response)
    }


}



impl Actor for InitializationActor {
    type Context = Context<Self>;
}

/// Handle `InitializedNotification` messages received via the TopicRegistryActor.
impl Handler<TopicMessage> for InitializationActor {
    type Result = Result<Value, ()>;

    fn handle(&mut self, msg: TopicMessage, _ctx: &mut Self::Context) -> Self::Result {
        if msg.topic == InitializedNotification::METHOD {
            tracing::info!("Received InitializedNotification");

            Ok(json!({ "status": "initialized notification received" }))
        } else {
            Err(())
        }
    }
}
