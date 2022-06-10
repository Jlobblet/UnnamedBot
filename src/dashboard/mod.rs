#![cfg(feature = "dashboard")]
use anyhow::{Context, Result};
use core::default::Default;
use rillrate::prime::{Pulse, PulseOpts};
use serenity::prelude::TypeMapKey;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use sysinfo::{RefreshKind, System, SystemExt};
use tokio::sync::Mutex;

const PACKAGE: &str = "Unnamed Bot";
const DASHBOARD_STATS: &str = "Statistics";
const GROUP_LATENCY: &str = "Discord Latency";
const GROUP_DISCORD_STATS: &str = "Discord Stats";
const GROUP_SYSTEM_STATS: &str = "System Stats";

#[derive(Debug)]
pub struct DashboardComponents {
    pub ws_ping_pulse: Pulse,
    pub get_ping_pulse: Pulse,
    pub message_pulse: Pulse,
    pub load_one_pulse: Pulse,
    pub load_five_pulse: Pulse,
    pub load_fifteen_pulse: Pulse,
    pub ram_pulse: Pulse,
    pub message_count: AtomicU32,
    pub system_info: Mutex<System>,
}

pub struct DashboardComponentsContainer;

impl TypeMapKey for DashboardComponentsContainer {
    type Value = Arc<DashboardComponents>;
}

pub async fn init_dashboard() -> Result<Arc<DashboardComponents>> {
    rillrate::install("unnamed bot").context("Failed to install rillrate")?;
    let mut system_info = System::new_with_specifics(
        RefreshKind::new()
            .with_cpu()
            .with_memory()
            .with_components(),
    );
    system_info.refresh_all();
    let total_memory = system_info.total_memory() as f64;

    let ws_ping_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_LATENCY,
            "Websocket Ping Time",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(1800_u32)
            .min(0)
            .max(200)
            .suffix("ms".to_string())
            .divisor(1.0),
    );

    let get_ping_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_LATENCY,
            "Rest GET Ping Time",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(1800_u32)
            .min(0)
            .max(200)
            .suffix("ms".to_string())
            .divisor(1.0),
    );

    let message_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_DISCORD_STATS,
            "Messages sent",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(3600_u32)
            .min(0)
            .max(200)
            .higher(true)
            .divisor(1.0),
    );

    let load_one_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_SYSTEM_STATS,
            "System Load (1m)",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(300_u32)
            .min(0.0)
            .max(1.5)
            .higher(true),
    );

    let load_five_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_SYSTEM_STATS,
            "System Load (5m)",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(1800_u32)
            .min(0.0)
            .max(1.5)
            .higher(true),
    );

    let load_fifteen_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_SYSTEM_STATS,
            "System Load (15m)",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(3600_u32)
            .min(0.0)
            .max(1.5)
            .higher(true),
    );

    let ram_pulse = Pulse::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_SYSTEM_STATS,
            "System RAM Usage",
        ],
        Default::default(),
        PulseOpts::default()
            .retain(3600_u32)
            .min(0)
            .max(total_memory)
            .suffix("GB".to_string())
            .divisor(1_000_000.0),
    );

    let dashboard_components = Arc::new(DashboardComponents {
        ws_ping_pulse,
        get_ping_pulse,
        message_pulse,
        load_one_pulse,
        load_five_pulse,
        load_fifteen_pulse,
        ram_pulse,
        message_count: AtomicU32::new(0),
        system_info: Mutex::new(system_info),
    });

    Ok(dashboard_components)
}
