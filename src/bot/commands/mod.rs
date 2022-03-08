use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group]
#[commands(ping)]
pub(crate) struct General;

#[command]
pub(crate) async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "🏓").await?;
    Ok(())
}
