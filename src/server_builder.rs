use actix::{Actor, Addr};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::EnvFilter;
use crate::client::ClientRegistryActor;
use crate::mcp::InitializationActor;
use crate::messages::{StartTransport, StopTransport};
use crate::router::RouterServiceManager;
use crate::transport::transport_config::Config;
use crate::transport::{SseTransportActor, StdioTransportActor, WasiTransportActor};
use crate::utils::LogConfig;

pub const SERVER: &str = "Multi MCP Router Server";
pub const VERSION: &str = "0.1.0";
pub enum TransportActorEnum
{
    Sse(Addr<SseTransportActor>),
    Wasi(Addr<WasiTransportActor>),
    Stdio(Addr<StdioTransportActor>),
}
pub struct McpServer{
    router_service_manager: Option<RouterServiceManager>,
    transport_config: Option<Config>,
    log_config: Option<LogConfig>,
    transport: Option<TransportActorEnum>,
}

impl McpServer
{
    pub fn new() -> Self {
        Self {
            router_service_manager: None,
            transport_config: None,
            log_config: None,
            transport: None,
        }
    }


    pub fn stop(&self) {
        // Stop the server with the configured router and transport
        println!("Stopping MCP Server...");
        match &self.transport {
            Some(TransportActorEnum::Sse(transport_addr)) => {
                transport_addr.do_send(StopTransport); // Assuming TransportRequest has Stop variant
            },
            Some(TransportActorEnum::Wasi(transport_addr)) => {
                transport_addr.do_send(StopTransport); 
            },
            Some(TransportActorEnum::Stdio(transport_addr)) => {
                transport_addr.do_send(StopTransport);
            },
            None => {
                println!("No transport configured");
            }
        }
    }

    pub fn router_manager(mut self,  router_service_manager: RouterServiceManager) -> Self {
        self.router_service_manager = Some(router_service_manager);
        self
    }

    pub fn transport(mut self, transport_config: Config) -> Self {
        self.transport_config = Some(transport_config);
        self
    }

    pub fn with_logging(mut self, log_config: LogConfig) -> Self {
        let file_appender = RollingFileAppender::new(Rotation::DAILY, log_config.clone().log_dir, log_config.clone().log_file);
        
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(log_config.level.into()))
            .with_writer(file_appender)
            .with_target(false)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .init();
        self.log_config = Some(log_config.clone());
        self
    }

    pub fn start(mut self) -> std::result::Result<Self, std::string::String> {
        
        if self.router_service_manager.is_none() || self.transport_config.is_none() {
            return Err("Missing required configuration".to_string());
        }
        let router_registry = self.router_service_manager.as_ref().unwrap().get_registry();
        let list_prompts_actor = self.router_service_manager.as_ref().unwrap().get_list_prompts();
        let list_tools_actor = self.router_service_manager.as_ref().unwrap().get_list_tools();
        let list_resources_actor = self.router_service_manager.as_ref().unwrap().get_list_resources();
        let client_registry = ClientRegistryActor::new().start();
        let transport_config = self.transport_config.as_ref().unwrap().clone();

        let transport = match transport_config {
            Config::Sse(sse_transport) => {
                let addr = SseTransportActor::new(
                    sse_transport, 
                    client_registry, 
                    router_registry, 
                    InitializationActor::new(),
                    list_prompts_actor,
                    list_tools_actor,
                    list_resources_actor,
                ).start();
                TransportActorEnum::Sse(addr)
            },
            Config::Wasi(wasi_transport_config) => {
                let addr = WasiTransportActor::new(
                    wasi_transport_config, 
                    client_registry, 
                        router_registry,
                    list_prompts_actor,
                list_tools_actor,
            list_resources_actor).unwrap().start();
                TransportActorEnum::Wasi(addr)
            },
            Config::Stdio(_stdio_transport_config) => {
                let addr = StdioTransportActor::new(router_registry).start();
                TransportActorEnum::Stdio(addr)
            },
        };

        self.transport = Some(transport);

        match &self.transport {
            Some(TransportActorEnum::Sse(transport_addr)) => {
                // Start the SseTransportActor
                transport_addr.do_send(StartTransport);
            },
            Some(TransportActorEnum::Wasi(transport_addr)) => {
                // Start the WasiTransportActor
                transport_addr.do_send(StartTransport); 
            },
            Some(TransportActorEnum::Stdio(transport_addr)) => {
                // Start the StdioTransportActor
                transport_addr.do_send(StartTransport);
            },
            None => {
                println!("No transport configured");
            }
        }

        Ok(self)

    }
}
