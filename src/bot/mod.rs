use crate::bot::commands::GENERAL_GROUP;
use crate::Config;
use eyre::{Context, Result};
use serenity::framework::{Framework, StandardFramework};
use serenity::Client;

pub mod commands;
pub mod handler;

pub fn default_framework(cfg: &Config) -> StandardFramework {
    StandardFramework::new()
        .configure(|c| c.prefix(cfg.prefix.clone()))
        .group(&GENERAL_GROUP)
}

pub async fn default_client<F>(cfg: &Config, framework: F) -> Result<Client>
where
    F: Framework + Send + Sync + 'static,
{
    Client::builder(&cfg.discord_api_key)
        .event_handler(handler::Handler)
        .framework(framework)
        .await
        .context("Failed to build client")
}
