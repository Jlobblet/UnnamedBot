use crate::PgConnectionContainer;
use caseless::Caseless;
use diesel::PgConnection;
use serenity::client::Context;
use std::sync::Arc;
use tokio::sync::Mutex;
use unicode_normalization::UnicodeNormalization;

pub fn compatibility_case_fold(s: &str) -> String {
    s.nfd()
        .default_case_fold()
        .nfkd()
        .default_case_fold()
        .nfkd()
        .collect()
}

pub async fn get_conn(ctx: &Context) -> Arc<Mutex<PgConnection>> {
    let data = ctx.data.read().await;
    data.get::<PgConnectionContainer>().unwrap().clone()
}

pub async fn user_is_administrator_in_guild(ctx: &Context, guild_id: u64, user_id: u64) -> bool {
    match ctx.http.get_member(guild_id, user_id).await {
        Ok(m) => m
            .permissions(ctx)
            .await
            .map(|p| p.administrator())
            .unwrap_or(false),
        _ => false,
    }
}
