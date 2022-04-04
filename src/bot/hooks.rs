use crate::models::alias::Alias;
use crate::PgConnectionContainer;
use log::{error, info};
use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Deref;

#[hook]
pub(crate) async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    info!(
        "Calling command '{}' (invoked by {} at {})",
        cmd_name,
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

#[hook]
pub async fn unrecognised_command(ctx: &Context, msg: &Message, unrecognised_command_name: &str) {
    let conn = {
        let data = ctx.data.read().await;
        data.get::<PgConnectionContainer>().unwrap().clone()
    };
    let conn = conn.lock().await;

    let result = Alias::search(conn.deref(), unrecognised_command_name);
    if let Ok(Some(a)) = result {
        let reply = msg.reply(ctx, a.command_text).await;
        if let Err(e) = reply {
            error!(
                "Failed to send alias message for '{}': {:?}",
                unrecognised_command_name, e
            )
        }
    } else if let Err(e) = result {
        error!(
            "Failed to search database for alias '{}': {:?}",
            unrecognised_command_name, e
        )
    }
}
