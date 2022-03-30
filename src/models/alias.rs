use crate::schema::aliases;
use diesel::{Insertable, Queryable};
use std::fmt::Debug;

#[derive(Clone, Debug, Insertable, Queryable)]
#[table_name = "aliases"]
pub struct Alias {
    user_id: i64,
    pub command_name: String,
    pub command_text: String,
}

impl Alias {
    pub fn new(user_id: u64, command_name: String, command_text: String) -> Self {
        Self {
            user_id: user_id as i64,
            command_name,
            command_text,
        }
    }

    #[inline]
    pub fn user_id(&self) -> u64 {
        self.user_id as u64
    }
}
