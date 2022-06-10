use anyhow::Context;
use reqwest::get;
use serde_derive::Deserialize;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::Context as SContext;

#[derive(Deserialize)]
struct HyenaUrl {
    url: String,
}

#[group]
#[commands(yeen, cowsay)]
pub struct Hyena;

#[command]
async fn yeen(ctx: &SContext, msg: &Message) -> CommandResult {
    let response = get("https://api.yeen.land").await?;
    let HyenaUrl { url } = response.json().await?;
    msg.reply(ctx, url).await?;
    Ok(())
}

#[command]
async fn cowsay(ctx: &SContext, msg: &Message, args: Args) -> CommandResult {
    let input = if let Some(m) = &msg.referenced_message {
        m.content.as_str()
    } else {
        args.rest()
    };

    let input = input.replace("```", "");

    let output = std::process::Command::new("cowsay")
        .args(input.split(' '))
        .output()
        .context("Failed to execute `cowsay`")?;
    if !output.status.success() {
        return Err(CommandError::from(format!(
            "`cowsay` exited with {}",
            output.status
        )));
    }
    let text = std::str::from_utf8(&output.stdout).context("Failed to parse cowsay output")?;
    let content = format!("```\n{}\n```", text);
    msg.reply(ctx, content).await?;
    Ok(())
}
