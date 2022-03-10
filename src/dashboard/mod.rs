use anyhow::{Context, Result};
use rillrate::prime::table::{Col, Row};
use rillrate::prime::{Pulse, PulseOpts, Table, TableOpts};
use serenity::framework::standard::CommandGroup;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

const PACKAGE: &str = "Unnamed Bot";
const DASHBOARD_STATS: &str = "Statistics";
const GROUP_LATENCY: &str = "Discord Latency";
const GROUP_COMMAND_COUNT: &str = "Command Invocation Count";

#[derive(Debug, Copy, Clone)]
pub(crate) struct CommandUsageValue {
    pub(crate) index: u64,
    pub(crate) use_count: usize,
}

#[derive(Debug)]
pub(crate) struct DashboardComponents {
    pub(crate) ws_ping_pulse: Pulse,
    pub(crate) get_ping_pulse: Pulse,
    pub(crate) command_usage_table: Table,
    pub(crate) command_usage_values: Mutex<HashMap<&'static str, CommandUsageValue>>,
}

pub(crate) struct DashboardComponentsContainer;

impl TypeMapKey for DashboardComponentsContainer {
    type Value = Arc<DashboardComponents>;
}

pub(crate) async fn init_dashboard(
    groups: &[&'static CommandGroup],
) -> Result<Arc<DashboardComponents>> {
    rillrate::install("unnamed bot").context("Failed to install rillrate")?;

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
        command_usage_table,
        command_usage_values: Mutex::new(command_usage_values),
    });

    Ok(dashboard_components)
}
