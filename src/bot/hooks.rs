use log::{error, info};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[hook]
pub(crate) async fn after(
    _ctx: &Context,
    _msg: &Message,
    command_name: &str,
    command_result: CommandResult,
) {
    match command_result {
        Ok(()) => info!("Processed command '{}'", command_name),
        Err(e) => error!("Command '{}' returned error {:?}", command_name, e),
    }
}
