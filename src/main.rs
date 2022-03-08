use crate::config::Config;
use eyre::{Context, Result};
use log::{debug, info, Level};
use try_traits::default::TryDefault;

mod bot;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let level = if cfg!(debug_assertions) { Level::Debug } else { Level::Info };
    simple_logger::init_with_level(level)?;
    info!("Starting bot");
    debug!("Creating config");
    let cfg = Config::try_default()?;
    debug!("Creating framework");
    let framework = bot::default_framework(&cfg);
    debug!("Creating client");
    let mut client = bot::default_client(&cfg, framework)
        .await
        .context("Failed to get client")?;
    info!("Connecting to Discord");
    client.start().await.context("Failed to start client")?;

    Ok(())
}
