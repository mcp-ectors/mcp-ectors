
# MCP SSE Server (mcp-ectors)

The **MCP SSE Server**, or **mcp-ectors** for short, is an enterprise-ready, high-performance server designed to enable seamless integration and interaction between large language models (LLMs) and various tools, resources, and workflow prompts. This powerful server acts as the bridge, much like a **USB** interface, for LLMs to gain access to multiple capabilities, enabling agents and agentic AI. Built using **Rust** and **actors**, it is optimized for performance and scalability, making it a great fit for enterprise environments.

> **Note**: The name *mcp-ectors* comes from "MCP Enterprise Actors Server", and has nothing to do with the creator's last name, despite how it might sound 😉

## Key Features
- **High Performance**: Built with **actors** and **Rust**, the server ensures high scalability and concurrency.
- **MCP as the USB for LLMs**: Enables access to tools, resources, and workflow prompts through a clean API.
- **Reuse Connections**: Unlike other MCP servers, mcp-ectors allows multiple routers to be deployed on the same connection, simplifying architecture and resource management.
- **Multiple Routers**: Register multiple routers and utilize them dynamically through the Router Service Manager.

## Getting Started

### Prerequisites
To get started with mcp-ectors, make sure you have the following installed:
- **Rust** (via [rust-lang](https://www.rust-lang.org/))
- **Cargo** (comes with Rust)
- **Cargo run** for running the application.

### Running the Server
1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/mcp-ectors.git
   cd mcp-ectors
   ```

2. **Run the server**:
   After cloning the repository, navigate to the project folder and run the server with:
   ```bash
   cargo run
   ```
   The server will start on **http://localhost:8080/sse**.

3. **Start with the Goose Desktop**:
   In **Goose Desktop** (a companion tool), you can add extensions, choose **SSE** as the transport, and use the following URL:
   ```http
   http://localhost:8080/sse
   ```

### Using the Counter Example
1. After running the server, in the Goose Desktop application you can ask to increment the **counter** or get the current value.
   
2. Try also Hello World greet.

### Create New MCP Routers
MCP-ectors enables you to create and register new routers through the **Router Trait**. To add a new router, implement the `Router` trait for your new router, following the examples of the existing **CounterRouter** or **HelloWorldRouter**.

#### Example:
```rust
use mcp_ectors::router::Router;

pub struct MyNewRouter;

impl Router for MyNewRouter {
    fn handle_request(&self, request: Request) -> Response {
        // Implement your router's logic here
    }
}
```

### Registering Routers
In **MCP**, tools, resources, and prompts are registered as `routerid_tool`, `routerid_prompt`, and `routerid_resource` to keep everything well-organized. The Router Service Manager allows these elements to be dynamically added.

#### Example Registration:
```rust
let counterid = "counter".to_string();
let counter_router = Box::new(CounterRouter::new());
router_manager.register_router::<CounterRouter>(counterid, counter_router).await.expect("router could not be registered");

let hwid = "helloworld".to_string();
let hw_router = Box::new(HelloWorldRouter::new());
router_manager.register_router::<HelloWorldRouter>(hwid, hw_router).await.expect("router could not be registered");
```

### Architecture Overview

1. **Server Builder**:
   - The `server_builder` determines the transport layer. Currently, **SSE** is supported. Future versions will include **stdio** and **wasi** transport.
   - For now, the server supports **SSE** for communication between the client and the server.

2. **Router Service Manager**:
   - The `RouterServiceManager` is responsible for registering multiple routers and ensuring that each router can handle requests without the need for new connections.
   - This architecture allows you to deploy several routers with the same connection, making the system highly efficient and scalable.

3. **Log Configuration**:
   - The server can be configured to store logs in specific directories and set custom log levels for monitoring and debugging.
   - The configuration can be customized using the `LogConfig` struct.

4) **Transport Actor**:  
   Any transport used in the MCP server needs to implement the **transport actor**. The **TransportActorTrait** ensures that transports handle requests and messages correctly:
   - **TransportRequest**: Allows new `JsonRPCMessages` to be sent to the client. It is used for communication between the server and the client through the transport layer.
   - **StartTransport**: This message starts the transport, initializing any necessary background tasks, such as setting up connections or waiting for incoming messages.
   - **StopTransport**: This message stops the transport, gracefully shutting down any active connections or tasks tied to the transport layer.

5) **Router**:  
   The **Router** is a trait that any router needs to implement, closely linked to the MCP standard. The **RouterServiceManager** uses the **RouterRegistry** to register new routes. New routes are embedded inside a **RouterActor**, which manages the communication for specific functionality. For instance, when a **tools/call** request is made, the actor holding the router will respond with the appropriate action. This setup allows the server to dynamically respond to a variety of requests by leveraging multiple routers.

6) **Standard Actors**:  
   MCP has a set of standard actors that implement basic functions such as initialization and managing tools, prompts, and resources. These standard actors are responsible for handling initialization requests and responding to list requests for tools, prompts, and resources. This makes it easier to interact with these essential components, providing a uniform and standardized method of retrieving and managing the core assets across different routers.


#### Example Log Configuration:
```rust
let log_config = LogConfig {
    log_dir: "logs".to_string(),
    log_file: "server.log".to_string(),
    level: Level::INFO,
};
```

### Future Development
- **MCP Protocol**: The basics have been implemented but notifications are still missing. Also oAuth, secrets management,... are on the roadmap.
- **Transport Extensions**: Currently, the server supports **SSE** transport, with plans to add **stdio** and **wasi** in the future.
- **Help Wanted**: Contributions are welcome! If you have expertise in other transports like **WASI** or **stdio**, feel free to submit a PR.

## Conclusion
The **MCP SSE Server (mcp-ectors)** is built for high performance, scalability, and ease of use. With **actors**, **Rust**, and a clean architecture for managing multiple routers, it makes working with LLMs, tools, and resources effortless. Whether you're a researcher, developer, or AI enthusiast, mcp-ectors will help you integrate LLMs with the tools and resources you need for advanced agentic AI workflows.

---

Feel free to **contribute**, **test**, and **expand** the system for your enterprise use cases. You can rely on mcp-ectors to power your next-generation AI applications. If your company wants to create custom routers or partner, please reach out to [Maarten Ectors](https://linkedin.com/in/mectors).
 
