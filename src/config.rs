use figment::{Figment, providers::{Toml}};
use figment::providers::{Env, Format, Json, Yaml};
use serde_derive::{Serialize, Deserialize};
use eyre::{Context, Result};
use try_traits::default::TryDefault;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) discord_api_key: String,
}

impl TryDefault for Config {
    type Error = eyre::Error;

    fn try_default() -> Result<Self> {
        Figment::new()
            .merge(Toml::file("UnnamedBot.toml"))
            .merge(Env::prefixed("UNNAMEDBOT_"))
            .merge(Json::file("UnnamedBot.json"))
            .merge(Yaml::file("UnnamedBot.yaml"))
            .merge(Yaml::file("UnnamedBot.yml"))
            .extract()
            .context("Failed to load configuration")
    }
}
