use super::DB;
use crate::schema::users;
use anyhow::{anyhow, Context, Result};
use chrono_tz::Tz;
use diesel::{insert_into, Connection, Insertable, QueryDsl, Queryable, RunQueryDsl};
use std::fmt::Debug;

#[derive(Clone, Debug, Queryable, Insertable)]
#[table_name = "users"]
struct NewUser {
    user_id: i64,
    timezone: Option<String>,
}

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

    pub fn get_user<C>(conn: &C, id: u64) -> Result<Self>
    where
        C: Connection<Backend = DB>,
    {
        use crate::schema::users::dsl::*;
        Ok(users
            .find(id as i64)
            .get_result::<NewUser>(conn)
            .with_context(|| anyhow!("Could not find user {}", id))?
            .into())
    }

    pub fn create_user<C>(conn: &C, id: u64) -> Result<Self>
    where
        C: Connection<Backend = DB>,
    {
        use crate::schema::users::dsl::*;
        let user = User::new(id);
        insert_into(users)
            .values(NewUser::from(user))
            .execute(conn)
            .with_context(|| anyhow!("Failed to create new user {}", id))?;
        Ok(user)
    }

    pub fn get_or_create_user<C>(conn: &C, id: u64) -> Result<Self>
    where
        C: Connection<Backend = DB>,
    {
        Self::get_user(conn, id)
            .or_else(|_| Self::create_user(conn, id))
            .with_context(|| anyhow!("Failed to get or create user {}", id))
    }
}

impl From<User> for NewUser {
    fn from(user: User) -> Self {
        Self {
            user_id: user.user_id as i64,
            timezone: user.timezone.map(|tz| tz.name().to_string()),
        }
    }
}

impl From<NewUser> for User {
    fn from(new_user: NewUser) -> Self {
        Self {
            user_id: new_user.user_id as u64,
            timezone: new_user.timezone.and_then(|tz| tz.parse::<Tz>().ok()),
        }
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

impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.user_id == other.user_id
    }
}

impl Eq for User {}
