pub mod hyena;
pub mod image;

pub use self::image::IMAGE_GROUP;
pub use hyena::HYENA_GROUP;

use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[group]
#[commands(ping)]
pub(crate) struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "🏓").await?;
    Ok(())
}
