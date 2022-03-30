#[macro_use]
extern crate diesel;

use crate::bot::commands::{ALIAS_GROUP, GENERAL_GROUP, HYENA_GROUP, IMAGE_GROUP};
#[cfg(feature = "dashboard")]
use crate::bot::ShardManagerContainer;
use crate::config::Config;
#[cfg(feature = "dashboard")]
use crate::dashboard::DashboardComponentsContainer;
use crate::database::{establish_connection, PgConnectionContainer};
use anyhow::{Context, Result};
use flexi_logger::LogSpecification;
use log::{debug, info};
#[cfg(feature = "dashboard")]
use std::sync::Arc;
use tokio::sync::Mutex;
use try_traits::default::TryDefault;

mod bot;
mod config;
mod dashboard;
mod database;
mod models;
mod schema;

#[tokio::main]
async fn main() -> Result<()> {
    flexi_logger::Logger::with(LogSpecification::env_or_parse("warn, unnamed_bot=debug")?)
        .format_for_files(flexi_logger::detailed_format)
        .adaptive_format_for_stderr(flexi_logger::AdaptiveFormat::Custom(
            uncoloured_format,
            coloured_format,
        ))
        .adaptive_format_for_stderr(flexi_logger::AdaptiveFormat::Custom(
            uncoloured_format,
            coloured_format,
        ))
        .start()?;
    info!("Starting bot");

    debug!("Creating config");
    let cfg = Config::try_default()?;

    let groups = [&GENERAL_GROUP, &HYENA_GROUP, &IMAGE_GROUP, &ALIAS_GROUP];

    debug!("Creating framework");
    let framework = bot::default_framework(&cfg, &groups);

    debug!("Creating client");
    let mut builder = bot::default_client_builder(&cfg, framework)
        .await
        .context("Failed to get client builder")?;

    #[cfg(feature = "dashboard")]
    {
        debug!("Initialising rillrate");
        let dashboard_components = dashboard::init_dashboard(&groups).await?;
        builder = builder.type_map_insert::<DashboardComponentsContainer>(dashboard_components);
    }

    let pg_connection = Arc::new(Mutex::new(establish_connection(&cfg)?));
    builder = builder.type_map_insert::<PgConnectionContainer>(pg_connection);

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

fn uncoloured_format(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] [{} {}:{}] {} {}",
        now.format(flexi_logger::TS_DASHES_BLANK_COLONS_DOT_BLANK),
        record.module_path().unwrap_or("<unnamed>"),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        record.level(),
        &record.args().to_string(),
    )
}

fn coloured_format(
    w: &mut dyn std::io::Write,
    now: &mut flexi_logger::DeferredNow,
    record: &log::Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{}] [{} {}:{}] {} {}",
        flexi_logger::style(level)
            .paint(now.format(flexi_logger::TS_DASHES_BLANK_COLONS_DOT_BLANK)),
        flexi_logger::style(level).paint(record.module_path().unwrap_or("<unnamed>")),
        record.file().unwrap_or("<unnamed>"),
        record.line().unwrap_or(0),
        flexi_logger::style(level).paint(level.to_string()),
        flexi_logger::style(level).paint(&record.args().to_string())
    )
}
