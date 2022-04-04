use crate::{models, schema, PgConnectionContainer};
use diesel::{insert_into, PgConnection, RunQueryDsl};
use log::debug;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Deref;
use std::sync::Arc;

#[group]
#[commands(alias)]
pub(crate) struct Alias;

#[command]
#[sub_commands(add, remove)]
async fn alias(_ctx: &Context, _msg: &Message, mut _args: Args) -> CommandResult {
    Ok(())
}

#[command]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.quoted().single::<String>()?;
    let command_text = args.rest().to_string();
    let GuildId(guild_id) = msg.guild_id.ok_or("Must be used in a server")?;

    let conn = get_conn(ctx).await;
    let conn = conn.lock().await;

    let user = models::user::User::get_or_create(conn.deref(), msg.author.id.0)?;
    let alias = models::alias::Alias::new(user.user_id, guild_id, command_name, command_text);

    let query_result = {
        use schema::aliases::dsl::*;
        insert_into(aliases).values(alias).execute(conn.deref())
    };

    let message = match query_result {
        Ok(_) => "Successfully added alias".to_string(),
        Err(e) => format!("Failed to add alias: {}", e),
    };

    msg.reply(&ctx, message).await?;

    Ok(())
}

#[command]
async fn remove(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.quoted().single::<String>()?;

    if !args.is_empty() {
        debug!(
            "Ignoring extra text in command alias_remove: {}",
            args.rest()
        );
    }

    let conn = get_conn(ctx).await;
    let conn = conn.lock().await;

    let UserId(author_id) = msg.author.id;

    let alias = models::alias::Alias::search(conn.deref(), &command_name)?;

    let response =
        if let Some(a) = alias {
            let GuildId(guild_id) = msg
                .guild_id
                .ok_or("Must be used in the guild where the alias was added")?;

            let authorised = a.guild_id == guild_id
                && (a.user_id == author_id
                    || user_is_administrator_in_guild(ctx, guild_id, msg.author.id.0).await);

            if authorised {
                a.delete(conn.deref())?;
                "Successfully deleted alias"
            } else {
                "You are not the owner of this alias or an administrator"
            }
        } else {
            "Could not find alias"
        };

    msg.reply(&ctx, response).await?;

    Ok(())
}

async fn get_conn(ctx: &Context) -> Arc<Mutex<PgConnection>> {
    let data = ctx.data.read().await;
    data.get::<PgConnectionContainer>().unwrap().clone()
}

async fn user_is_administrator_in_guild(ctx: &Context, guild_id: u64, user_id: u64) -> bool {
    match ctx.http.get_member(guild_id, user_id).await {
        Ok(m) => m
            .permissions(ctx)
            .await
            .map(|p| p.administrator())
            .unwrap_or(false),
        _ => false,
    }
}
