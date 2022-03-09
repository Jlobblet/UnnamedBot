use reqwest::get;
use serde_derive::Deserialize;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::CommandResult;
use serenity::model::prelude::*;
use serenity::prelude::*;

#[derive(Deserialize)]
struct HyenaUrl {
    url: String,
}

#[group]
#[commands(yeen)]
pub(crate) struct Hyena;

#[command]
async fn yeen(ctx: &Context, msg: &Message) -> CommandResult {
    let response = get("https://api.yeen.land").await?;
    let HyenaUrl { url } = response.json().await?;
    msg.reply(ctx, url).await?;
    Ok(())
}
