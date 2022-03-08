use eyre::{Context, Result};
use figment::providers::{Env, Format, Json, Toml, Yaml};
use figment::Figment;
use serde_derive::Deserialize;
use try_traits::default::TryDefault;

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub discord_api_key: String,
    pub prefix: String,
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
