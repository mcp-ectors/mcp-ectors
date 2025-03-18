use actix::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

// A message to subscribe to a topic.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Subscribe {
    pub topic: String,
    pub addr: Recipient<TopicMessage>,
}

// A message to publish to a topic.
#[derive(Message)]
#[rtype(result = "Vec<TopicResponse>")] // Aggregated responses.
pub struct Publish {
    pub topic: String,
    pub payload: Value, // Or use a JSON value.
}

// A message that subscribers receive.
#[derive(Message)]
#[rtype(result = "Result<serde_json::Value, ()>")]
pub struct TopicMessage {
    pub topic: String,
    pub payload: Value,
}

// The response type that subscribers return.
#[derive(Debug)]
pub struct TopicResponse {
    pub data: String,
}

pub struct TopicRegistryActor {
    subscriptions: HashMap<String, Vec<Recipient<TopicMessage>>>,
}

impl TopicRegistryActor {
    pub fn new() -> Self {
        Self {
            subscriptions: HashMap::new(),
        }
    }
}

impl Actor for TopicRegistryActor {
    type Context = Context<Self>;
}

// Handle subscription requests.
impl Handler<Subscribe> for TopicRegistryActor {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _ctx: &mut Self::Context) -> Self::Result {
        self.subscriptions
            .entry(msg.topic)
            .or_default()
            .push(msg.addr);
    }
}

// Handle publish requests.
impl Handler<Publish> for TopicRegistryActor {
    type Result = ResponseActFuture<Self, Vec<TopicResponse>>;

    fn handle(&mut self, msg: Publish, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(subscribers) = self.subscriptions.get(&msg.topic) {
            // Map each subscriber to a normal future (without .into_actor(self))
            let futs: Vec<_> = subscribers.iter().map(|addr| {
                addr.send(TopicMessage {
                    topic: msg.topic.clone(),
                    payload: msg.payload.clone(),
                })
            }).collect();

            // Create an async block that awaits all futures.
            let fut = async move {
                // Use futures::future::join_all to await all futures.
                let results = futures::future::join_all(futs).await;
                // Filter out errors and collect successful responses.
                let responses: Vec<TopicResponse> = results.into_iter()
                .map(|res| {
                    let value = res.map_err(|_| "expected Ok(serde_json::Value) but got Err")
                        .expect("Expected Ok(serde_json::Value) but got Err");
                    TopicResponse { data: format!("{:?}", value) }
                })
                .collect();
                responses
            };

            // Wrap the entire async block into an ActorFuture.
            Box::pin(actix::fut::wrap_future(fut))
        } else {
            Box::pin(actix::fut::wrap_future(async { Vec::new() }))
        }
    }
}
