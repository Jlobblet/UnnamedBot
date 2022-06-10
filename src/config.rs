use anyhow::{Context, Result};
use dotenv::dotenv;
use figment::providers::{Env, Format, Json, Toml, Yaml};
use figment::Figment;
use serde_derive::Deserialize;
use try_traits::default::TryDefault;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub discord_api_key: String,
    pub prefix: String,
    pub database_url: String,
}

impl TryDefault for Config {
    type Error = anyhow::Error;

    fn try_default() -> Result<Self> {
        dotenv().ok();

        Figment::new()
            .merge(Toml::file("UnnamedBot.toml"))
            .merge(Env::prefixed("UNNAMEDBOT_"))
            .merge(Env::raw().only(&["DATABASE_URL"]))
            .merge(Json::file("UnnamedBot.json"))
            .merge(Yaml::file("UnnamedBot.yaml"))
            .merge(Yaml::file("UnnamedBot.yml"))
            .extract()
            .context("Failed to load configuration")
    }
}
