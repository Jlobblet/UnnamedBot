use crate::{DashboardComponentsContainer, ShardManagerContainer};
use log::info;
use serenity::client::bridge::gateway::ShardId;
use serenity::client::Context;
use serenity::model::prelude::*;
use serenity::{async_trait, client::EventHandler};
use std::time::Duration;
use tokio::time::{interval, Instant};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache is ready");
        tokio::spawn(async move {
            let components = {
                let data = ctx.data.read().await;
                data.get::<DashboardComponentsContainer>().unwrap().clone()
            };

            let mut interval = interval(Duration::from_millis(42500));

            loop {
                interval.tick().await;
                let get_latency = {
                    let now = Instant::now();
                    // don't care about the result, just want to block for how long it takes
                    // to measure latency
                    let _ = reqwest::get("https://discordapp.com/api/v9/gateway").await;
                    now.elapsed().as_millis() as f64
                };
                components.get_ping_pulse.push(get_latency);

                let ws_latency = {
                    let data = ctx.data.read().await;
                    let shard_manager = data.get::<ShardManagerContainer>().unwrap();

                    let manager = shard_manager.lock().await;
                    let runners = manager.runners.lock().await;

                    let runner = runners.get(&ShardId(ctx.shard_id)).unwrap();

                    if let Some(d) = runner.latency {
                        d.as_millis() as f64
                    } else {
                        f64::NAN
                    }
                };

                components.ws_ping_pulse.push(ws_latency);
            }
        });
    }
}
