use serenity::{async_trait, client::EventHandler};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {}
