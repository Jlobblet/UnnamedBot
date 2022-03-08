use crate::config::Config;
use eyre::{Context, Result};
use try_traits::default::TryDefault;

mod bot;
mod config;

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::try_default()?;
    let framework = bot::default_framework(&cfg);
    let mut client = bot::default_client(&cfg, framework)
        .await
        .context("Failed to get client")?;
    client.start().await.context("Failed to start client")?;

    Ok(())
}
