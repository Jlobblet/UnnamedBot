extern crate core;

use crate::bot::commands::{GENERAL_GROUP, HYENA_GROUP, IMAGE_GROUP};
#[cfg(feature = "dashboard")]
use crate::bot::ShardManagerContainer;
use crate::config::Config;
#[cfg(feature = "dashboard")]
use crate::dashboard::DashboardComponentsContainer;
use anyhow::{Context, Result};
use log::{debug, info};
#[cfg(feature = "dashboard")]
use std::sync::Arc;
use try_traits::default::TryDefault;

mod bot;
mod config;
mod dashboard;

#[tokio::main]
async fn main() -> Result<()> {
    flexi_logger::Logger::try_with_str("warn")?.start()?;
    info!("Starting bot");

    debug!("Creating config");
    let cfg = Config::try_default()?;

    let groups = [&GENERAL_GROUP, &HYENA_GROUP, &IMAGE_GROUP];

    debug!("Creating framework");
    let framework = bot::default_framework(&cfg, &groups);

    #[cfg(feature = "dashboard")]
    debug!("Initialising rillrate");
    #[cfg(feature = "dashboard")]
    let dashboard_components = dashboard::init_dashboard(&groups).await?;

    debug!("Creating client");
    let builder = bot::default_client_builder(&cfg, framework)
        .await
        .context("Failed to get client builder")?;

    #[cfg(feature = "dashboard")]
    let builder = builder.type_map_insert::<DashboardComponentsContainer>(dashboard_components);

    let mut client = builder.await.context("Failed to build client")?;

    #[cfg(feature = "dashboard")]
    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    info!("Connecting to Discord");
    client.start().await.context("Failed to start client")?;

    Ok(())
}
