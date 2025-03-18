
use std::collections::HashMap;
use mcp_spec::protocol::JsonRpcMessage;
use rand::random;
use actix::prelude::*;

/// Message to register a new client
#[derive(Message)]
#[rtype(result = "u64")]
pub struct RegisterClient {
    pub recipient: Recipient<ClientMessage>,
}

/// Message to remove a client
#[derive(Message)]
#[rtype(result = "()")]
pub struct DeregisterClient {
    pub client_id: u64,
}

/// Message to send a notification to a specific client
#[derive(Message)]
#[rtype(result = "Result<(), ()>")]
pub struct NotifyClient {
    pub client_id: u64,
    pub message: JsonRpcMessage,
}

/// Message to broadcast a message to all clients
#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage {
    pub message: JsonRpcMessage,
}

/// Message wrapper for client communication
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientRegistryMessage(pub JsonRpcMessage);

/// Actor that manages registered clients
pub struct ClientRegistryActor {
    clients: HashMap<u64, Recipient<ClientMessage>>,
}

impl ClientRegistryActor {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }
}

impl Actor for ClientRegistryActor {
    type Context = Context<Self>;
}

impl Handler<RegisterClient> for ClientRegistryActor {
    type Result = u64;

    fn handle(&mut self, msg: RegisterClient, _ctx: &mut Self::Context) -> Self::Result {
        let client_id: u64 = random();
        info!("New client {} registered", client_id);
        self.clients.insert(client_id, msg.recipient);
        client_id
    }
}

impl Handler<DeregisterClient> for ClientRegistryActor {
    type Result = ();

    fn handle(&mut self, msg: DeregisterClient, _ctx: &mut Self::Context) -> Self::Result {
        self.clients.remove(&msg.client_id);
    }
}

use actix::{fut::wrap_future, Actor, Message, Recipient};
use tracing::info;

use crate::messages::ClientMessage;

impl Handler<NotifyClient> for ClientRegistryActor {
    type Result = ResponseActFuture<Self, Result<(), ()>>;

    fn handle(&mut self, msg: NotifyClient, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(recipient) = self.clients.get(&msg.client_id) {
            let fut = recipient.send(ClientMessage(msg.message.clone()));
            Box::pin(wrap_future(async move { fut.await.map_err(|_| ()) }))
        } else {
            Box::pin(wrap_future(async { Err(()) }))
        }
    }
}


impl Handler<BroadcastMessage> for ClientRegistryActor {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, _ctx: &mut Self::Context) -> Self::Result {
        for (_, recipient) in self.clients.iter() {
            let _ = recipient.do_send(ClientMessage(msg.message.clone()));
        }
    }
}
