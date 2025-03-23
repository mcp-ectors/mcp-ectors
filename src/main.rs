
use mcp_ectors::examples::{CounterRouter, HelloWorldRouter};
use mcp_ectors::router::RouterServiceManager;
use mcp_ectors::transport::transport_config::Config;
use mcp_ectors::utils::LogConfig;
use mcp_ectors::McpServer;
use tracing::info;
use tokio::signal;
use tracing::Level;
use mcp_ectors::transport::sse_transport_actor::SseTransportConfig;


#[actix::main]
async fn main() {
    let log_config = LogConfig {
        log_dir: "logs".to_string(),
        log_file: "server.log".to_string(),
        level: Level::INFO,
    };

    info!("Starting MCP Server...");
     // Set up SSE transport configuration.
     let config = SseTransportConfig {
        port: 8080,
        tls_cert: None,
        tls_key: None,
        log_dir: "logs".into(),
        log_file: "sse.log".into(),
    };

    let wasm_path = "./wasm";

    let mut router_manager = RouterServiceManager::default(Some(wasm_path.to_string())).await;
    // ✅ Register router
    let counter_id = "counter".to_string();
    let counter_router = Box::new(CounterRouter::new());
    router_manager.register_router::<CounterRouter>(counter_id, counter_router).await.expect("router could not be registered");
    let hw_id = "helloworld".to_string();
    let hw_router = Box::new(HelloWorldRouter::new());
    router_manager.register_router::<HelloWorldRouter>(hw_id, hw_router).await.expect("router could not be registered");


    // ✅ Start MCP Server
    let server = McpServer::new()
        .router_manager(router_manager)
        .transport(Config::Sse(config)) // Fluent API for transport
        .with_logging(log_config) // Optionally add logging
        .start()
        .unwrap();


    // Graceful shutdown handling.
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c signal");
        info!("Shutdown signal received, exiting...");
    };

    // Wait until a shutdown signal is received.
    tokio::select! {
        _ = shutdown_signal => {
            info!("Shutting down MCP Server...");
            let _ = server.stop();
        }
    }
}
