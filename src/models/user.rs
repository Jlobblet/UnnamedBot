use super::DB;
use crate::schema::users;
use anyhow::{anyhow, Context, Result};
use chrono_tz::Tz;
use diesel::{
    insert_into, Connection, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl,
};
use std::fmt::Debug;

#[derive(Copy, Clone, Debug)]
pub struct User {
    pub user_id: u64,
    pub timezone: Option<Tz>,
}

impl User {
    pub fn new(id: u64) -> Self {
        Self {
            user_id: id,
            timezone: None,
        }
    }

    pub fn new_with_timezone(id: u64, timezone: Tz) -> Self {
        Self {
            user_id: id,
            timezone: Some(timezone),
        }
    }

    pub fn get<C>(conn: &C, id: u64) -> Result<Self>
    where
        C: Connection<Backend = DB>,
    {
        use crate::schema::users::dsl::*;
        users
            .find(id as i64)
            .get_result::<User>(conn)
            .with_context(|| anyhow!("Could not find user {}", id))
    }

    pub fn create<C>(conn: &C, id: u64) -> Result<Self>
    where
        C: Connection<Backend = DB>,
    {
        use crate::schema::users::dsl::*;
        let user = User::new(id);
        insert_into(users)
            .values(user)
            .execute(conn)
            .with_context(|| anyhow!("Failed to create new user {}", id))?;
        Ok(user)
    }

    pub fn get_or_create<C>(conn: &C, id: u64) -> Result<Self>
    where
        C: Connection<Backend = DB>,
    {
        Self::get(conn, id)
            .or_else(|_| Self::create(conn, id))
            .with_context(|| anyhow!("Failed to get or create user {}", id))
    }
}

impl Queryable<users::SqlType, DB> for User {
    type Row = (i64, Option<String>);

    fn build(row: Self::Row) -> Self {
        User {
            user_id: row.0 as u64,
            timezone: row.1.and_then(|s| s.parse::<Tz>().ok()),
        }
    }
}

impl Insertable<users::table> for User {
    type Values = <(
        Option<diesel::dsl::Eq<users::user_id, i64>>,
        Option<diesel::dsl::Eq<users::timezone, String>>,
    ) as Insertable<users::table>>::Values;

    fn values(self) -> Self::Values {
        (
            Some(users::user_id.eq(self.user_id as i64)),
            self.timezone
                .map(|x| users::timezone.eq(x.name().to_string())),
        )
            .values()
    }
}

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl Eq for User {}
