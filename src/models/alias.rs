use super::DB;
use crate::schema::aliases;
use crate::util::compatibility_case_fold;
use anyhow::{anyhow, Context, Result};
use diesel::{
    Connection, ExpressionMethods, Insertable, OptionalExtension, PgTextExpressionMethods,
    QueryDsl, Queryable, RunQueryDsl,
};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Alias {
    alias_id: Option<i64>,
    pub user_id: u64,
    pub guild_id: u64,
    pub command_name: String,
    pub command_text: String,
}

impl Alias {
    pub fn new(user_id: u64, guild_id: u64, command_name: String, command_text: String) -> Self {
        Self {
            alias_id: None,
            user_id,
            guild_id,
            command_name,
            command_text,
        }
    }

    pub fn search<C>(conn: &C, search_term: &str, id: u64) -> Result<Option<Alias>>
    where
        C: Connection<Backend = DB>,
    {
        let id = id as i64;
        use crate::schema::aliases::dsl::*;
        let search_term = compatibility_case_fold(search_term);

        aliases
            .filter(guild_id.eq(id))
            .filter(command_name.ilike(&search_term))
            .first(conn)
            .optional()
            .with_context(|| anyhow!("Failed to find alias from search term {}", &search_term))
    }

    pub fn delete<C>(self, conn: &C) -> Result<()>
    where
        C: Connection<Backend = DB>,
    {
        use crate::schema::aliases::dsl::*;

        if let Some(id) = self.alias_id {
            diesel::delete(aliases.filter(alias_id.eq(id)))
                .execute(conn)
                .map(|_| ())
                .with_context(|| anyhow!("Failed to delete alias with ID {}", id))
        } else {
            Err(anyhow!("Alias to delete had no ID"))
        }
    }
}

impl Queryable<aliases::SqlType, DB> for Alias {
    type Row = (i64, i64, i64, String, String);

    fn build(row: Self::Row) -> Self {
        let (alias_id, user_id, guild_id, command_name, command_text) = row;
        let user_id = user_id as u64;
        let guild_id = guild_id as u64;

        Alias {
            alias_id: Some(alias_id),
            user_id,
            guild_id,
            command_name,
            command_text,
        }
    }
}

impl Insertable<aliases::table> for Alias {
    type Values = <(
        diesel::dsl::Eq<aliases::user_id, i64>,
        diesel::dsl::Eq<aliases::guild_id, i64>,
        diesel::dsl::Eq<aliases::command_name, String>,
        diesel::dsl::Eq<aliases::command_text, String>,
    ) as Insertable<aliases::table>>::Values;

    fn values(self) -> Self::Values {
        (
            aliases::user_id.eq(self.user_id as i64),
            aliases::guild_id.eq(self.guild_id as i64),
            aliases::command_name.eq(self.command_name),
            aliases::command_text.eq(self.command_text),
        )
            .values()
    }
}

impl<'a> Insertable<aliases::table> for &'a Alias {
    type Values = <(
        diesel::dsl::Eq<aliases::user_id, i64>,
        diesel::dsl::Eq<aliases::guild_id, i64>,
        diesel::dsl::Eq<aliases::command_name, &'a String>,
        diesel::dsl::Eq<aliases::command_text, &'a String>,
    ) as Insertable<aliases::table>>::Values;

    fn values(self) -> Self::Values {
        (
            aliases::user_id.eq(self.user_id as i64),
            aliases::guild_id.eq(self.guild_id as i64),
            aliases::command_name.eq(&self.command_name),
            aliases::command_text.eq(&self.command_text),
        )
            .values()
    }
}
