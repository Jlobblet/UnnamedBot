pub mod alias;
pub mod hyena;
pub mod image;

pub use self::image::IMAGE_GROUP;
pub use alias::ALIAS_GROUP;
pub use hyena::HYENA_GROUP;

use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[macro_export]
macro_rules! parse_args {
    ($args:expr, $($t:ty),*) => {
        (
            $($args.single::<$t>()?,)*
        )
    };
}

#[group]
#[commands(ping, test)]
pub struct General;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.reply(ctx, "🏓").await?;
    Ok(())
}

#[command]
async fn test(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let (i, f, s, b) = parse_args!(args.quoted(), i32, f32, String, bool);
    msg.reply(ctx, format!("{}, {}, {}, {}", i, f, s, b))
        .await?;
    Ok(())
}
