use crate::bot::commands::{GENERAL_GROUP, HYENA_GROUP, IMAGE_GROUP};
use crate::Config;
use eyre::{Context, Result};
use serenity::framework::{Framework, StandardFramework};
use serenity::Client;

pub(crate) mod commands;
pub(crate) mod handler;
pub(crate) mod hooks;

pub(crate) fn default_framework(cfg: &Config) -> StandardFramework {
    StandardFramework::new()
        .configure(|c| c.prefix(cfg.prefix.clone()))
        .after(hooks::after)
        .group(&GENERAL_GROUP)
        .group(&IMAGE_GROUP)
        .group(&HYENA_GROUP)
}

pub(crate) async fn default_client<F>(cfg: &Config, framework: F) -> Result<Client>
where
    F: Framework + Send + Sync + 'static,
{
    Client::builder(&cfg.discord_api_key)
        .event_handler(handler::Handler)
        .framework(framework)
        .await
        .context("Failed to build client")
}
