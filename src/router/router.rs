


use std::{future::Future, pin::Pin};

use mcp_spec::{handler::ResourceError, prompt::Prompt, protocol::{CallToolResult, GetPromptResult, PromptsCapability, ReadResourceResult, ResourcesCapability, ServerCapabilities, ToolsCapability}, Resource, Tool, ToolError};
use serde_json::Value;
pub type ResponseFuture<I> = Pin<Box<dyn Future<Output = I>>>;
pub trait Router
where
Self: Send + Sync + 'static,
{
    fn name(&self) -> String;
    fn instructions(&self) -> String;
    fn capabilities(&self) -> ServerCapabilities;
    fn list_tools(&self) -> Vec<Tool>;
    fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> ResponseFuture<Result<CallToolResult, ToolError>>;
    fn list_resources(&self) -> Vec<Resource>;
    fn read_resource(
        &self,
        uri: &str,
    ) -> ResponseFuture<Result<ReadResourceResult, ResourceError>>;
    fn list_prompts(&self) -> Vec<Prompt>;
    fn get_prompt(&self, prompt_name: &str) -> ResponseFuture<Result<GetPromptResult, ResourceError>>;
}


/// Builder for configuring and constructing capabilities
pub struct CapabilitiesBuilder {
    tools: Option<ToolsCapability>,
    prompts: Option<PromptsCapability>,
    resources: Option<ResourcesCapability>,
}

impl Default for CapabilitiesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CapabilitiesBuilder {
    pub fn new() -> Self {
        Self {
            tools: None,
            prompts: None,
            resources: None,
        }
    }

    /// Add multiple tools to the router
    pub fn with_tools(mut self, list_changed: bool) -> Self {
        self.tools = Some(ToolsCapability {
            list_changed: Some(list_changed),
        });
        self
    }

    /// Enable prompts capability
    pub fn with_prompts(mut self, list_changed: bool) -> Self {
        self.prompts = Some(PromptsCapability {
            list_changed: Some(list_changed),
        });
        self
    }

    /// Enable resources capability
    pub fn with_resources(mut self, subscribe: bool, list_changed: bool) -> Self {
        self.resources = Some(ResourcesCapability {
            subscribe: Some(subscribe),
            list_changed: Some(list_changed),
        });
        self
    }

    /// Build the router with automatic capability inference
    pub fn build(self) -> ServerCapabilities {
        // Create capabilities based on what's configured
        ServerCapabilities {
            tools: self.tools,
            prompts: self.prompts,
            resources: self.resources,
        }
    }
}
