#[cfg(feature = "dashboard")]
use crate::{DashboardComponentsContainer, ShardManagerContainer};
#[cfg(feature = "dashboard")]
use log::info;
#[cfg(feature = "dashboard")]
use serenity::client::bridge::gateway::ShardId;
#[cfg(feature = "dashboard")]
use serenity::client::Context;
#[cfg(feature = "dashboard")]
use serenity::model::prelude::*;
use serenity::{async_trait, client::EventHandler};
use std::sync::atomic::Ordering;
#[cfg(feature = "dashboard")]
use std::time::Duration;
use sysinfo::SystemExt;
#[cfg(feature = "dashboard")]
use tokio::time::{interval, Instant};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[cfg(feature = "dashboard")]
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache is ready");
        let ctx_clone = ctx.clone();
        tokio::spawn(async move {
            let components = {
                let data = ctx_clone.data.read().await;
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
                    let data = ctx_clone.data.read().await;
                    let shard_manager = data.get::<ShardManagerContainer>().unwrap();

                    let manager = shard_manager.lock().await;
                    let runners = manager.runners.lock().await;

                    let runner = runners.get(&ShardId(ctx_clone.shard_id)).unwrap();

                    if let Some(d) = runner.latency {
                        d.as_millis() as f64
                    } else {
                        f64::NAN
                    }
                };

                components.ws_ping_pulse.push(ws_latency);
            }
        });

        let ctx_clone = ctx;
        tokio::spawn(async move {
            let components = {
                let data = ctx_clone.data.write().await;
                data.get::<DashboardComponentsContainer>().unwrap().clone()
            };

            let mut interval = interval(Duration::from_millis(5000));

            loop {
                interval.tick().await;
                let messages = components.message_count.load(Ordering::Acquire);
                components.message_count.store(0, Ordering::Release);
                components.message_pulse.push(messages as f64);

                let (mem, one, five, fifteen) = {
                    let mut sys = components.system_info.lock().await;
                    sys.refresh_all();
                    let load = sys.load_average();
                    (sys.used_memory() as f64, load.one, load.five, load.fifteen)
                };

                components.ram_pulse.push(mem);
                components.load_one_pulse.push(one);
                components.load_five_pulse.push(five);
                components.load_fifteen_pulse.push(fifteen);
            }
        });
    }

    #[cfg(feature = "dashboard")]
    async fn message(&self, ctx: Context, _new_message: Message) {
        let components = {
            let data = ctx.data.read().await;
            data.get::<DashboardComponentsContainer>().unwrap().clone()
        };
        components.message_count.fetch_add(1, Ordering::Relaxed);
    }
}
