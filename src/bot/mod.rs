use crate::Config;
use anyhow::Result;
use serenity::client::bridge::gateway::ShardManager;
use serenity::client::ClientBuilder;
use serenity::framework::standard::CommandGroup;
use serenity::framework::{Framework, StandardFramework};
use serenity::prelude::TypeMapKey;
use serenity::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

pub(crate) mod commands;
pub(crate) mod handler;
pub(crate) mod hooks;

pub(crate) struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

pub(crate) fn default_framework(
    cfg: &Config,
    groups: &[&'static CommandGroup],
) -> impl Framework {
    let mut framework = StandardFramework::new()
        .configure(|c| c.prefix(cfg.prefix.clone()))
        .before(hooks::before)
        .after(hooks::after)
        .unrecognised_command(hooks::unrecognised_command);

    for group in groups {
        framework.group_add(group);
    }

    framework
}

pub(crate) async fn default_client_builder<F>(
    cfg: &Config,
    framework: F,
) -> Result<ClientBuilder<'_>>
where
    F: Framework + Send + Sync + 'static,
{
    Ok(Client::builder(&cfg.discord_api_key)
        .event_handler(handler::Handler)
        .framework(framework))
}
