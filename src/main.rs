
use std::fs;
use std::path::Path;

use clap::{Arg, Command};
use mcp_ectors::router::RouterServiceManager;
use mcp_ectors::server_builder::VERSION;
use mcp_ectors::transport::transport_config::Config;
use mcp_ectors::utils::LogConfig;
use mcp_ectors::McpServer;
use tracing::info;
use tokio::signal;
use tracing::Level;
use mcp_ectors::transport::sse_transport_actor::SseTransportConfig;

const LOGS_DIR: &str = "logs";
const LOGS_FILE: &str = "server.log";
const PORT: &str = "8080";
const WASM_DIR: &str = "./wasm";

#[actix::main]
async fn main() {
    let matches = Command::new("MCP-Ectors Server")
        .version(VERSION)
        .author("Maarten Ectors <maarten@ectors.com>")
        .about("MCP Server with Wasm Management")
        .subcommand_required(false)
        .subcommand(
            Command::new("start")
                .arg(Arg::new("log_dir")
                    .long("log_dir")
                    .default_value(LOGS_DIR)
                    .help("Directory for log files"))
                .arg(Arg::new("log_file")
                    .long("log_file")
                    .default_value(LOGS_FILE)
                    .help("Log file name"))
                .arg(Arg::new("port")
                    .long("port")
                    .default_value(PORT)
                    .help("Port for the server"))
                .arg(Arg::new("wasm_path")
                    .long("wasm_path")
                    .default_value(WASM_DIR)
                    .help("Path to WASM files"))
                .arg(Arg::new("tls_cert")
                    .long("tls_cert")
                    .value_name("CERT")
                    .value_parser(clap::value_parser!(String)) // Set value parser
                    .help("TLS certificate file"))
                .arg(Arg::new("tls_key")
                    .long("tls_key")
                    .value_name("KEY")
                    .value_parser(clap::value_parser!(String)) // Set value parser
                    .help("TLS key file"))
        )
        .subcommand(
            Command::new("login")
                .about("Log into the remote store and authenticate"),
        )
        .subcommand(
            Command::new("pull")
                .about("Pull a specific wasm from the store")
                .arg(Arg::new("wasm_name").required(true)),
        )
        .subcommand(
            Command::new("search")
                .about("Search for a specific MCP")
                .arg(Arg::new("terms").required(true)),
        )
        .subcommand(
            Command::new("publish")
                .about("Publish a wasm to the general repo"),
        )
        .get_matches();

    match matches.subcommand() {
        None => {
            start_server(LOGS_DIR.to_string(), LOGS_FILE.to_string(), WASM_DIR.to_string(), PORT.parse().unwrap(), None, None).await;
        },
        Some(("start", sub_m)) => {
            
            let log_dir = sub_m.get_one::<String>("log_dir").unwrap().to_string();
            let log_file = sub_m.get_one::<String>("log_file").unwrap().to_string();
            let port = sub_m.get_one::<String>("port").unwrap().parse::<u16>().unwrap();
            let wasm_path = sub_m.get_one::<String>("wasm_path").unwrap().to_string();
            let tls_cert = sub_m.get_one::<String>("tls_cert").map(|s| s.to_string());
            let tls_key = sub_m.get_one::<String>("tls_key").map(|s| s.to_string());

            start_server(log_dir, log_file, wasm_path, port, tls_cert, tls_key).await;
        }
        Some(("login", _)) => {
            // Implement OAuth login flow here
            println!("Logging in to the remote store...");
            // Use reqwest or another library to handle OAuth and authenticate
        }
        Some(("pull", sub_m)) => {
            let wasm_name = sub_m.get_one::<String>("wasm_name").unwrap();
            // Implement pull logic to download the specific Wasm file
            println!("Pulling Wasm: {}", wasm_name);
        }
        Some(("search", sub_m)) => {
            let search_terms = sub_m.get_one::<String>("terms").unwrap();
            // Implement search logic for MCPs
            println!("Searching for: {}", search_terms);
        }
        Some(("publish", _)) => {
            // Implement logic to publish Wasm to the repository
            println!("Publishing Wasm...");
        }
        _ => {}
    }
}

async fn start_server(log_dir: String, log_file: String, wasm_path: String, port: u16, tls_cert: Option<String>, tls_key: Option<String>) {
    let log_config = LogConfig {
        log_dir,
        log_file,
        level: Level::INFO,
    };

    let wasm_path_dir = Path::new(&wasm_path);

    // Check if the path exists, if not, create it
    if !wasm_path_dir.exists() {
        match fs::create_dir_all(wasm_path_dir) {
            Ok(_) => println!("Created wasm directory at {:?}", wasm_path_dir),
            Err(e) => eprintln!("Failed to create wasm directory at {:?}: {}", wasm_path_dir, e),
        }
    }

    let config = SseTransportConfig {
        port,
        tls_cert,
        tls_key,
        log_dir: log_config.log_dir.clone(),
        log_file: log_config.log_file.clone(),
    };

    let router_manager = RouterServiceManager::default(Some(wasm_path)).await;
    //let counter_id = "counter".to_string();
    //let counter_router = Box::new(CounterRouter::new());
    //router_manager.register_router::<CounterRouter>(counter_id, counter_router).await.expect("router could not be registered");

    let server = McpServer::new()
        .router_manager(router_manager)
        .transport(Config::Sse(config))
        .with_logging(log_config)
        .start()
        .unwrap();

    // Graceful shutdown handling
    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for ctrl_c signal");
        info!("Shutdown signal received, exiting...");
    };

    tokio::select! {
        _ = shutdown_signal => {
            info!("Shutting down MCP Server...");
            let _ = server.stop();
        }
    }
}