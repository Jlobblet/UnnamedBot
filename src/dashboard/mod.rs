#![cfg(feature = "dashboard")]
use anyhow::{Context, Result};
use core::default::Default;
use rillrate::prime::table::{Col, Row};
use rillrate::prime::{Pulse, PulseOpts, Table, TableOpts};
use serenity::framework::standard::CommandGroup;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU32;
use sysinfo::{RefreshKind, System, SystemExt};
use tokio::sync::Mutex;

const PACKAGE: &str = "Unnamed Bot";
const DASHBOARD_STATS: &str = "Statistics";
const GROUP_LATENCY: &str = "Discord Latency";
const GROUP_DISCORD_STATS: &str = "Discord Stats";
const GROUP_COMMAND_COUNT: &str = "Command Invocation Count";
const GROUP_SYSTEM_STATS: &str = "System Stats";

#[derive(Debug, Copy, Clone)]
pub(crate) struct CommandUsageValue {
    pub(crate) index: u64,
    pub(crate) use_count: usize,
}

#[derive(Debug)]
pub(crate) struct DashboardComponents {
    pub(crate) ws_ping_pulse: Pulse,
    pub(crate) get_ping_pulse: Pulse,
    pub(crate) message_pulse: Pulse,
    pub(crate) load_one_pulse: Pulse,
    pub(crate) load_five_pulse: Pulse,
    pub(crate) load_fifteen_pulse: Pulse,
    pub(crate) ram_pulse: Pulse,
    pub(crate) command_usage_table: Table,
    pub(crate) command_usage_values: Mutex<HashMap<&'static str, CommandUsageValue>>,
    pub(crate) message_count: AtomicU32,
    pub(crate) system_info: Mutex<System>,
}

pub(crate) struct DashboardComponentsContainer;

impl TypeMapKey for DashboardComponentsContainer {
    type Value = Arc<DashboardComponents>;
}

pub(crate) async fn init_dashboard(
    groups: &[&'static CommandGroup],
) -> Result<Arc<DashboardComponents>> {
    rillrate::install("unnamed bot").context("Failed to install rillrate")?;
    let mut system_info = System::new_with_specifics(RefreshKind::new().with_cpu().with_memory().with_components());
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

    let mut command_usage_values = HashMap::new();

    let command_usage_table = Table::new(
        [
            PACKAGE,
            DASHBOARD_STATS,
            GROUP_COMMAND_COUNT,
            "Command Usage",
        ],
        Default::default(),
        TableOpts::default().columns(vec![
            (0, "Command Name".to_string()),
            (1, "Number of Uses".to_string()),
        ]),
    );

    for (index, command) in groups.iter().flat_map(|g| g.options.commands).enumerate() {
        let index = index as u64;
        let row = Row(index);
        command_usage_table.add_row(row);
        command_usage_table.set_cell(row, Col(0), command.options.names[0]);
        command_usage_table.set_cell(row, Col(1), 0_usize);
        command_usage_values.insert(
            command.options.names[0],
            CommandUsageValue {
                index,
                use_count: 0,
            },
        );
    }

    let dashboard_components = Arc::new(DashboardComponents {
        ws_ping_pulse,
        get_ping_pulse,
        message_pulse,
        load_one_pulse,
        load_five_pulse,
        load_fifteen_pulse,
        ram_pulse,
        command_usage_table,
        command_usage_values: Mutex::new(command_usage_values),
        message_count: AtomicU32::new(0),
        system_info: Mutex::new(system_info),
    });

    Ok(dashboard_components)
}
