use crate::{models, schema, PgConnectionContainer};
use diesel::{insert_into, RunQueryDsl};
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::ops::Deref;

#[group]
#[commands(alias_add)]
pub(crate) struct Alias;

#[command]
async fn alias_add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let command_name = args.quoted().single::<String>()?;
    let command_text = args.rest().to_string();

    let conn = {
        let data = ctx.data.read().await;
        data.get::<PgConnectionContainer>().unwrap().clone()
    };

    let conn = conn.lock().await;

    let user = models::user::User::get_or_create_user(conn.deref(), msg.author.id.0)?;
    let alias = models::alias::Alias::new(user.user_id, command_name, command_text);

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
