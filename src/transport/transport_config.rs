use super::{sse_transport_actor::SseTransportConfig, stdio_transport_actor::StdioTransportConfig, wasi_transport_actor::WasiTransportConfig};

#[derive(Clone,Debug)]
pub enum Config {
    Sse(SseTransportConfig),
    Stdio(StdioTransportConfig),
    Wasi(WasiTransportConfig),
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub config: Config,
}

impl TransportConfig {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn display_config(&self) {
        match &self.config {
            Config::Sse(sse) => println!("Config Sse: {:?}", sse),
            Config::Stdio(stdio) => println!("Config Stdio: {:?}", stdio),
            Config::Wasi(wasi) => println!("Config Wasi: {:?}", wasi),
        }
    }
}