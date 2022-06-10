use crate::Config;
use anyhow::{anyhow, Context, Result};
use diesel::{Connection, PgConnection};
use serenity::prelude::TypeMapKey;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PgConnectionContainer;

impl TypeMapKey for PgConnectionContainer {
    type Value = Arc<Mutex<PgConnection>>;
}

pub fn establish_connection(config: &Config) -> Result<PgConnection> {
    PgConnection::establish(&config.database_url)
        .with_context(|| anyhow!("Failed to establish connection to {}", config.database_url))
}
