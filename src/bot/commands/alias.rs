use crate::{models, schema, util};
use diesel::{insert_into, RunQueryDsl};
use log::debug;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Deref;

#[group]
#[commands(alias)]
pub struct Alias;

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

    let message = {
        let conn = util::get_conn(ctx).await;
        let conn = conn.lock().await;

        let user = models::user::User::get_or_create(conn.deref(), msg.author.id.0)?;
        let alias = models::alias::Alias::new(user.user_id, guild_id, command_name, command_text);

        let query_result = {
            use schema::aliases::dsl::*;
            insert_into(aliases).values(alias).execute(conn.deref())
        };

        match query_result {
            Ok(_) => "Successfully added alias".to_owned(),
            Err(e) => format!("Failed to add alias: {}", e),
        }
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

    let GuildId(guild_id) = msg.guild_id.ok_or("Must be used in a server")?;

    let response = {
        let conn = util::get_conn(ctx).await;
        let conn = conn.lock().await;

        let UserId(author_id) = msg.author.id;

        let alias = models::alias::Alias::search(conn.deref(), &command_name, guild_id)?;

         if let Some(a) = alias {
            let GuildId(guild_id) = msg
                .guild_id
                .ok_or("Must be used in the guild where the alias was added")?;

            let authorised = a.guild_id == guild_id
                && (a.user_id == author_id
                    || util::user_is_administrator_in_guild(ctx, guild_id, msg.author.id.0).await);

            if authorised {
                a.delete(conn.deref())?;
                "Successfully deleted alias"
            } else {
                "You are not the owner of this alias or an administrator"
            }
        } else {
             "Could not find alias"
        }
    };

    msg.reply(&ctx, response).await?;

    Ok(())
}
