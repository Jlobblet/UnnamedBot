#[cfg(feature = "dashboard")]
use crate::dashboard::{CommandUsageValue, DashboardComponentsContainer};
use log::{error, info};
#[cfg(feature = "dashboard")]
use rillrate::prime::table::{Col, Row};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[hook]
pub(crate) async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    #[cfg(feature = "dashboard")]
    {
        let components = {
            let data = ctx.data.read().await;
            data.get::<DashboardComponentsContainer>().unwrap().clone()
        };

        let CommandUsageValue { index, use_count } = {
            let mut count_write = components.command_usage_values.lock().await;
            let mut value = count_write.get_mut(cmd_name).unwrap();
            value.use_count += 1;
            *value
        };

        components
            .command_usage_table
            .set_cell(Row(index), Col(1), use_count);
    }

    info!(
        "Calling command '{cmd_name}' (invoked by {} at {})",
        msg.author.tag(),
        msg.timestamp
    );

    true
}

#[hook]
pub(crate) async fn after(
    ctx: &Context,
    msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => info!("Processed command '{}'", command_name),
        Err(e) => {
            error!("Command '{}' returned error {:?}", command_name, e);
            if let Err(e) = msg.reply(ctx, format!("{}", e)).await {
                error!(
                    "Failed to send error message for command '{}': {:?}",
                    command_name, e
                );
            };
        }
    }
}
