use actix::prelude::*;
use actix_web_lab::sse::{Event, Data as SseData};
use mcp_spec::protocol::JsonRpcMessage;
use tokio::sync::mpsc;

/// Message to send data to a connected client via SSE
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientSessionMessage(pub JsonRpcMessage);

/// Manages an individual client session
pub struct ClientSessionActor {
    sender: mpsc::Sender<Event>, // SSE channel for pushing messages
}

impl ClientSessionActor {
    pub fn new(sender: mpsc::Sender<Event>) -> Self {
        Self { sender }
    }
}

impl Actor for ClientSessionActor {
    type Context = Context<Self>;
}

impl Handler<ClientSessionMessage> for ClientSessionActor {
    type Result = ();

    fn handle(&mut self, msg: ClientSessionMessage, _ctx: &mut Self::Context) {
        let data = serde_json::to_string(&msg.0).unwrap_or_else(|_| "{}".to_string());
        let event = Event::Data(SseData::new(data));

        // Try sending the SSE event
        let _ = self.sender.try_send(event);
    }
}
