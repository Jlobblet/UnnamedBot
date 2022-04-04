use super::DB;
use crate::schema::aliases;
use diesel::{ExpressionMethods, Insertable, Queryable};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Alias {
    pub user_id: u64,
    pub command_name: String,
    pub command_text: String,
}

impl Alias {
    pub fn new(user_id: u64, command_name: String, command_text: String) -> Self {
        Self {
            user_id,
            command_name,
            command_text,
        }
    }
}

impl Queryable<aliases::SqlType, DB> for Alias {
    type Row = (i64, i64, String, String);

    fn build(row: Self::Row) -> Self {
        let (_alias_id, user_id, command_name, command_text) = row;
        let user_id = user_id as u64;

        Alias {
            user_id,
            command_name,
            command_text,
        }
    }
}

impl Insertable<aliases::table> for Alias {
    type Values = <(
        Option<diesel::dsl::Eq<aliases::user_id, i64>>,
        Option<diesel::dsl::Eq<aliases::command_name, String>>,
        Option<diesel::dsl::Eq<aliases::command_text, String>>,
    ) as Insertable<aliases::table>>::Values;

    fn values(self) -> Self::Values {
        (
            Some(aliases::user_id.eq(self.user_id as i64)),
            Some(aliases::command_name.eq(self.command_name)),
            Some(aliases::command_text.eq(self.command_text)),
        )
            .values()
    }
}

impl<'a> Insertable<aliases::table> for &'a Alias {
    type Values = <(
        Option<diesel::dsl::Eq<aliases::user_id, i64>>,
        Option<diesel::dsl::Eq<aliases::command_name, &'a String>>,
        Option<diesel::dsl::Eq<aliases::command_text, &'a String>>,
    ) as Insertable<aliases::table>>::Values;

    fn values(self) -> Self::Values {
        (
            Some(aliases::user_id.eq(self.user_id as i64)),
            Some(aliases::command_name.eq(&self.command_name)),
            Some(aliases::command_text.eq(&self.command_text)),
        )
            .values()
    }
}
