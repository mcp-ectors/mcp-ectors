
#[cfg(test)]
#[path = "../tests/mock_router.rs"]
mod wasm_mock_router;

#[cfg(test)]
mod tests {
    //use std::{sync::Arc, thread, time::Duration};

    use mcp_ectors::router::router_registry::ROUTER_SEPERATOR;
    use mcp_ectors::router::RouterServiceManager;
    use mcp_ectors::server_builder::{SERVER, VERSION};
    use mcp_ectors::transport::sse_transport_actor::SseTransportConfig;
    use mcp_ectors::transport::transport_config::Config;
    use mcp_ectors::utils::LogConfig;
    use mcp_ectors::McpServer;
    use mcp_spec::prompt::PromptMessageContent;
    use mcp_spec::protocol::{Implementation, InitializeResult, PromptsCapability, ResourcesCapability, ServerCapabilities, ToolsCapability};
    use mcp_spec::{ResourceContents, Tool};
    use serde_json::{json, Value};
    use tokio::signal;
    use tokio::sync::mpsc::{self, UnboundedSender};
    use tokio::task::spawn_local;
    use tokio::time::Instant;
    use tokio::time::sleep;
    use tracing::{event, info, Level};
    use mcp_client::client::{ClientCapabilities, ClientInfo, McpClient, McpClientTrait};
    use mcp_client::{McpService, Transport as ClientTransport};
    use std::collections::{HashMap, HashSet};
    use std::time::Duration;

    use crate::wasm_mock_router::MockRouter;

    fn get_initialize_result() -> InitializeResult {
        InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                resources: Some(ResourcesCapability {
                    list_changed: Some(true),
                    subscribe: Some(true),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
                prompts: Some(PromptsCapability { list_changed: Some(true) },) 
            },
            server_info: Implementation {
                name: SERVER.to_string(),
                version: VERSION.to_string(),
            },
            instructions: Some(format!("Please initialize your session. A multi mcp router server allows many routers to be installed. You can see this in the name of prompts and tools. They are formatted routerid{}prompt_name or routerid{}tool_name. The same for resource which are routerid{}uri. To get a list of what this server offers call resources/read with uri system{}all to understand which tools, prompts and resources this multi mcp router server has installed and what they do.",ROUTER_SEPERATOR,ROUTER_SEPERATOR,ROUTER_SEPERATOR,ROUTER_SEPERATOR)),
        }
    }

    fn get_initial_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "tool1".to_string(),
                description: "Test tool 1".to_string(),
                input_schema: Value::String("input schema 1".to_string()),
            },
            Tool {
                name: "tool2".to_string(),
                description: "Test tool 2".to_string(),
                input_schema: Value::String("input schema 2".to_string()),
            },
        ]
    }

    async fn run_server(shutdown_receiver: tokio::sync::oneshot::Receiver<()>, wasm_path: Option<String>) {
        let log_config = LogConfig {
            log_dir: "logs".to_string(),
            log_file: "test-server.log".to_string(),
            level: Level::WARN,
        };
        
        info!("Starting MCP Server...");
        let config = SseTransportConfig {
            port: 3000,
            tls_cert: None,
            tls_key: None,
            log_dir: "logs".into(),
            log_file: "test-sse.log".into(),
        };

        let mut router_manager = RouterServiceManager::default(wasm_path).await;
        // ✅ Register router
        let mock_id = "mockrouter".to_string();
        let mock = MockRouter::new(get_initialize_result().clone(),get_initial_tools().clone());
        let _ = router_manager.register_router::<MockRouter>(mock_id, Box::new(mock)).await;
        

        let server = McpServer::new()
        .router_manager(router_manager)
        .transport(Config::Sse(config)) // Fluent API for transport
        .with_logging(log_config) // Optionally add logging
        .start()
        .unwrap();
        
        // Graceful shutdown handling.
        let ctrl_c_signal = async {
            
            info!("Shutdown signal received, exiting...");
            signal::ctrl_c().await
        };
        
        // Graceful shutdown handling.
        let shutdown_signal = async {
            shutdown_receiver.await
        };

        // Wait until a shutdown signal is received.
        tokio::select! {
            _ = ctrl_c_signal => {
                let _ = server.stop();
            },
            _ = shutdown_signal => {
                info!("Shutting down MCP Server...");
                let _ = server.stop();
            }
        }
        
    }


    #[tokio::test]
    async fn test_server_integration_with_mock_router() {
        // Create a channel to signal when the server should stop
        let (shutdown_trigger, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        let local = tokio::task::LocalSet::new();
        local.run_until(async {

            // Spawn the server in a background task
            let server_task = spawn_local(async {
                run_server(shutdown_receiver, None).await;
            });

            // Give the server some time to start.
            let _ = sleep(Duration::from_millis(200));


            let client_transport = mcp_client::SseTransport::new("http://localhost:3000/sse", HashMap::new());
            let handle = client_transport.start().await.unwrap();
            let service = McpService::with_timeout(handle, Duration::from_secs(3));
            //let service = McpService::new(handle);
            let mut client = McpClient::new(service);

            let server_info = client
                .initialize(
                    ClientInfo {
                        name: "test-client".into(),
                        version: "1.0.0".into(),
                    },
                    ClientCapabilities::default(),
                )
                .await;
            tracing::info!("Server info: {:?}",server_info);
            assert_eq!(server_info.unwrap(), get_initialize_result());


            let list_tools_req = client.list_tools(None).await;
            let list_tools = list_tools_req.unwrap();
            assert_eq!(list_tools.tools.len(), get_initial_tools().len());
            for expected in &get_initial_tools() {
                assert!(list_tools.tools.iter().any(|t| t.name == format!("mockrouter{}{}",ROUTER_SEPERATOR,expected.name)));
            }

            let tool_result = client
                .call_tool(
                    format!("mockrouter{}tool1",ROUTER_SEPERATOR).as_str(),
                    serde_json::json!({ "message": "Client with SSE transport - calling a tool" }),
                )
                .await.unwrap();
            tracing::info!("Tool result: {:?}", tool_result);
            assert_eq!(tool_result.is_error, Some(false));

            // 4. Assert that listing resources returns a non-empty list.
            let resources = client.list_resources(None).await.unwrap();
            tracing::info!("Resources: {:?}", resources);
            assert!(!resources.resources.is_empty());
            
            // 5. Assert that reading a specific resource returns a valid response.
            let resource = client.read_resource(format!("mockrouter{}echo://fixedresource",ROUTER_SEPERATOR).as_str()).await.unwrap();
            tracing::info!("Resource: {:?}", resource);
            if let Some(ResourceContents::TextResourceContents { text, .. }) = resource.contents.first() {
                assert_eq!(text, "expected resource value");
            } else {
                panic!("Expected a TextResourceContents variant");
            }

            // 6. Assert that listing prompts returns the expected prompt.
            let prompts= client.list_prompts(None).await.unwrap();
            tracing::info!("Prompts: {:?}", prompts);
            assert_eq!(prompts.prompts.len(), 1);
            let prompt = &prompts.prompts[0];
            assert_eq!(prompt.name, format!("mockrouter{}dummy_prompt",ROUTER_SEPERATOR));
            assert_eq!(prompt.description.as_ref().unwrap(), "A dummy prompt for testing");

            // 7. Test retrieving a prompt by name.
            let prompt_future = client.get_prompt(format!("mockrouter{}dummy_prompt",ROUTER_SEPERATOR).as_str(), json!({})).await;
            let prompt_response = prompt_future.unwrap().messages;
            if let PromptMessageContent::Text { text } = &prompt_response[0].content {
                assert_eq!(text, "dummy prompt response");
            } else {
                panic!("Expected a Text variant");
            }

            // Once the test logic is complete, trigger the shutdown.
            let _ = shutdown_trigger.send(());  // Wait for the shutdown signal
            tracing::info!("Test completed, server should stop now.");

            // Stop the server gracefully
            let _ = server_task.await;
        }).await;
    }

    #[tokio::test]
    async fn test_server_integration_with_wasm_mock_router() {
        // Create a channel to signal when the server should stop
        let (shutdown_trigger, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        let local = tokio::task::LocalSet::new();
        local.run_until(async {

            // Spawn the server in a background task
            let server_task = spawn_local(async {
                run_server(shutdown_receiver, Some("tests/wasm/target/wasm32-wasip2/debug".to_string())).await;
            });

            // Give the server some time to start.
            let _ = sleep(Duration::from_millis(200));


            let client_transport = mcp_client::SseTransport::new("http://localhost:3000/sse", HashMap::new());
            let handle = client_transport.start().await.unwrap();
            let service = McpService::with_timeout(handle, Duration::from_secs(3));
            //let service = McpService::new(handle);
            let mut client = McpClient::new(service);

            let server_info = client
                .initialize(
                    ClientInfo {
                        name: "test-client".into(),
                        version: "1.0.0".into(),
                    },
                    ClientCapabilities::default(),
                )
                .await;
            tracing::info!("Server info: {:?}",server_info);
            assert_eq!(server_info.unwrap(), get_initialize_result());


            let list_tools_req = client.list_tools(None).await;
            let list_tools = list_tools_req.unwrap();
            assert!(list_tools.tools.len() > 1);
            for expected in &get_initial_tools() {
                assert!(list_tools.tools.iter().any(|t| t.name == format!("mockrouter{}{}",ROUTER_SEPERATOR,expected.name)));
            }

            let tool_result = client
                .call_tool(
                    format!("mcpmockrouter{}tool1",ROUTER_SEPERATOR).as_str(),
                    serde_json::json!({ "message": "Client with SSE transport - calling a tool" }),
                )
                .await.unwrap();
            tracing::info!("Tool result: {:?}", tool_result);
            assert_eq!(tool_result.is_error, Some(false));

            // 4. Assert that listing resources returns a non-empty list.
            let resources = client.list_resources(None).await.unwrap();
            tracing::info!("Resources: {:?}", resources);
            assert!(!resources.resources.is_empty());
            
            // 5. Assert that reading a specific resource returns a valid response.
            let resource = client.read_resource(format!("mcpmockrouter{}echo://fixedresource",ROUTER_SEPERATOR).as_str()).await.unwrap();
            tracing::info!("Resource: {:?}", resource);
            if let Some(ResourceContents::TextResourceContents { text, .. }) = resource.contents.first() {
                assert_eq!(text, "expected resource value");
            } else {
                panic!("Expected a TextResourceContents variant");
            }

            // 6. Assert that listing prompts returns the expected prompt.
            let prompts= client.list_prompts(None).await.unwrap();
            tracing::info!("Prompts: {:?}", prompts);
            assert_eq!(prompts.prompts.len(), 2);
            let prompt = &prompts.prompts[0];
            assert_eq!(prompt.name, format!("mcpmockrouter{}dummy_prompt",ROUTER_SEPERATOR));
            assert_eq!(prompt.description.as_ref().unwrap(), "A dummy prompt for testing");

            // 7. Test retrieving a prompt by name.
            let prompt_future = client.get_prompt(format!("mcpmockrouter{}dummy_prompt",ROUTER_SEPERATOR).as_str(), json!({})).await;
            let prompt_response = prompt_future.unwrap().messages;
            if let PromptMessageContent::Text { text } = &prompt_response[0].content {
                assert_eq!(text, "dummy prompt response");
            } else {
                panic!("Expected a Text variant");
            }

            // Once the test logic is complete, trigger the shutdown.
            let _ = shutdown_trigger.send(());  // Wait for the shutdown signal
            tracing::info!("Test completed, server should stop now.");

            // Stop the server gracefully
            let _ = server_task.await;
        }).await;
    }


    
    #[tokio::test]
    async fn performance_test_with_unlimited_messaging() {
        // Create a channel to signal when the server should stop
        let (shutdown_trigger, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        let local = tokio::task::LocalSet::new();
        local.run_until(async {

            // Spawn the server in a background task
            let server_task = spawn_local(async {
                run_server(shutdown_receiver, None).await;
            });

            // Give the server some time to start.
            let _ = sleep(Duration::from_millis(1000));

            let time:usize = 5;
            // Test settings.
            let run_duration = Duration::from_secs(time.try_into().unwrap());
            let num_clients = 5;
        
            // Create an unbounded channel for performance messages.
            let (tx, mut rx) = mpsc::unbounded_channel::<PerfCallMessage>();
        
            // Spawn an aggregator task.
            let aggregator_handle = tokio::spawn(async move {
                // Use a HashSet to track active clients.
                let mut active_clients: HashSet<usize> = HashSet::new();
                let mut last_tool_calls = 0usize;
                let start_time = Instant::now();
        
                while start_time.elapsed() < run_duration {
                    // Sleep for 1 second.
                    sleep(Duration::from_secs(1)).await;
        
                    // Drain the channel.
                    let mut tool_calls_this_interval = 0;
                    while let Ok(msg) = rx.try_recv() {
                        match msg {
                            PerfCallMessage::ClientStart { client_id } => {
                                active_clients.insert(client_id);
                            }
                            PerfCallMessage::ClientEnd { client_id } => {
                                active_clients.remove(&client_id);
                            }
                            PerfCallMessage::ToolCall { _client_id: _ } => {
                                tool_calls_this_interval += 1;
                            }
                        }
                    }
                    info!(
                        "Active clients: {} | Tool calls in last second: {}",
                        active_clients.len(),
                        tool_calls_this_interval
                    );
                    last_tool_calls += tool_calls_this_interval;
                }
                last_tool_calls
            });
        
            // Spawn client tasks.
            let mut client_handles = Vec::new();
            for client_id in 0..num_clients {
                let tx_clone: UnboundedSender<PerfCallMessage> = tx.clone();
                let client_handle = tokio::spawn(async move {
                    // Send a start message.
                    let _ = tx_clone.send(PerfCallMessage::ClientStart { client_id });
                    
                    // Create a new client transport (simulate MCP client startup)
                    let client_transport = mcp_client::SseTransport::new("http://localhost:3000/sse", HashMap::new());
                    let transport_handle = match client_transport.start().await {
                        Ok(h) => h,
                        Err(e) => {
                            eprintln!("Client {} failed to start transport: {:?}", client_id, e);
                            let _ = tx_clone.send(PerfCallMessage::ClientEnd { client_id });
                            return;
                        }
                    };
                    let service = McpService::new(transport_handle);
                    let mut client = McpClient::new(service);
        
                    // Initialize the client once.
                    let init_result = client.initialize(
                        mcp_client::client::ClientInfo {
                            name: format!("client{}", client_id),
                            version: "1.0.0".into(),
                        },
                        mcp_client::client::ClientCapabilities::default(),
                    ).await;
                    if init_result.is_err() {
                        let _ = tx_clone.send(PerfCallMessage::ClientEnd { client_id });
                        return;
                    }
        
                    // List tools once.
                    if client.list_tools(None).await.is_err() {
                        let _ = tx_clone.send(PerfCallMessage::ClientEnd { client_id });
                        return;
                    }
        
                    // Repeatedly call the tool until the overall test duration elapses.
                    let start_time = Instant::now();
                    while start_time.elapsed() < run_duration {
                        let result = client.call_tool(
                            format!("mockrouter{}echo_tool",ROUTER_SEPERATOR).as_str(),
                            json!({ "message": "performance test" }),
                        ).await;
                        if result.is_err() {
                            // Optionally, you can send an error message.
                        } else {
                            // Send a tool call message on each successful call.
                            let _ = tx_clone.send(PerfCallMessage::ToolCall { _client_id: client_id });
                        }
                    }
        
                    // Send a client end message.
                    let _ = tx_clone.send(PerfCallMessage::ClientEnd { client_id });
                    let _ = client_transport.close().await;
                    
                });
                client_handles.push(client_handle);
            }
        
            // Drop the original sender so that the aggregator will eventually get a closed channel.
            drop(tx);
        
            // Wait for all client tasks to finish.
            for handle in client_handles {
                let _ = handle.await;
            }
        
            // Wait for the aggregator and get the total tool calls count.
            let total_tool_calls = aggregator_handle.await.unwrap();
            let calls_per_sec = total_tool_calls / time;
            event!(Level::ERROR,
                "Total tool calls performed over {} seconds: {} with {}/sec",
                time,
                total_tool_calls,
                calls_per_sec
            );
            assert!(calls_per_sec>1000);

            // Once the test logic is complete, trigger the shutdown.
            let _ = shutdown_trigger.send(());  // Wait for the shutdown signal
            tracing::info!("Test completed, server should stop now.");

            // Stop the server gracefully
            let _ = server_task.await;
        }).await;
    }
    

    #[derive(Debug)]
    enum PerfCallMessage {
        ClientStart { client_id: usize },
        ClientEnd { client_id: usize },
        ToolCall { _client_id: usize },
    }

    #[derive(Debug)]
    enum PerfConnectionMessage {
        ConnectionOpened { _client_id: usize },
        ConnectionClosed { _client_id: usize },
    }

    #[tokio::test]
    async fn performance_test_with_messaging() {
        // Create a channel to signal when the server should stop
        let (shutdown_trigger, shutdown_receiver) = tokio::sync::oneshot::channel::<()>();

        let local = tokio::task::LocalSet::new();
        local.run_until(async {

            // Spawn the server in a background task
            let server_task = spawn_local(async {
                run_server(shutdown_receiver, None).await;
            });

    
            // Give the server time to start.
            sleep(Duration::from_millis(200)).await;
     
            let time: usize = 5;
            // Test settings.
            let run_duration = Duration::from_secs(time.try_into().unwrap());
            let num_clients = 5;
            let num_tool_calls = 10; // configurable number of tool calls per cycle
    
            // Create an unbounded channel for performance messages.
            let (tx, mut rx) = mpsc::unbounded_channel::<PerfConnectionMessage>();
    
            // Aggregator task: every second, print the number of connection cycles.
            let aggregator_handle = tokio::spawn(async move {
                let mut total_connections = 0usize;
                let start_time = Instant::now();
                while start_time.elapsed() < run_duration {
                    sleep(Duration::from_secs(1)).await;
                    while let Ok(msg) = rx.try_recv() {
                        if let PerfConnectionMessage::ConnectionOpened { .. } = msg {
                            total_connections += 1;
                        }
                    }
                    info!("Total connections opened so far: {}", total_connections);
                }
                total_connections
            });
    
            // Spawn client tasks.
            let mut client_handles = Vec::new();
            for client_id in 0..num_clients {
                let tx_clone: UnboundedSender<PerfConnectionMessage> = tx.clone();
                let client_handle = tokio::spawn(async move {
                    let start_time = Instant::now();
                    while start_time.elapsed() < run_duration {
                        // Each cycle: open a connection.
                        let client_transport = mcp_client::SseTransport::new("http://localhost:3000/sse", HashMap::new());
                        match client_transport.start().await {
                            Ok(transport_handle) => {
                                let _ = tx_clone.send(PerfConnectionMessage::ConnectionOpened { _client_id: client_id });
                                let service = McpService::new(transport_handle);
                                let mut client = McpClient::new(service);
                                // Initialize the client.
                                if client.initialize(
                                    ClientInfo {
                                        name: format!("client{}", client_id),
                                        version: "1.0.0".into(),
                                    },
                                    ClientCapabilities::default(),
                                ).await.is_err() {
                                    let _ = tx_clone.send(PerfConnectionMessage::ConnectionClosed { _client_id: client_id });
                                    continue;
                                }
                                // Make a fixed number of tool calls.
                                for _ in 0..num_tool_calls {
                                    let _ = client.call_tool(format!("mockrouter{}echo_tool",ROUTER_SEPERATOR).as_str(), json!({ "message": "performance test" })).await;
                                }
                                // Connection cycle complete: close connection.
                                let _ = tx_clone.send(PerfConnectionMessage::ConnectionClosed { _client_id: client_id });

                                let _ = client_transport.close().await;
                            }
                            Err(e) => {
                                eprintln!("Client {} failed to start transport: {:?}", client_id, e);
                            }
                        }
                    }
                });
                client_handles.push(client_handle);
            }
    
            // Drop the original sender so the aggregator can eventually finish.
            drop(tx);
    
            // Wait for all client tasks to finish.
            for handle in client_handles {
                let _ = handle.await;
            }
    
            sleep(Duration::from_millis(200)).await;
            // Wait for the aggregator task to finish.
            let total_connections = aggregator_handle.await.unwrap();
            let calls_per_sec = total_connections / time;
            event!(Level::ERROR,
                "Total connections opened over {} seconds: {} at {} per second",
                run_duration.as_secs(),
                total_connections,
                calls_per_sec,
            );

            assert!(calls_per_sec>5);

            sleep(Duration::from_millis(200)).await;
            // Once the test logic is complete, trigger the shutdown.
            let _ = shutdown_trigger.send(());  // Wait for the shutdown signal
            tracing::info!("Test completed, server should stop now.");

            // Stop the server gracefully
            let _ = server_task.await;
        }).await;
    }

}
