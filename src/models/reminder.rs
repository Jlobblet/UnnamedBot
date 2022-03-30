use super::DB;
use crate::schema::reminders;
use chrono::{DateTime, TimeZone, Utc};
use diesel::{Insertable, Queryable};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Reminder<TZ>
where
    TZ: TimeZone,
{
    pub user_id: u64,
    pub reminder_time: DateTime<TZ>,
    pub reminder_text: String,
    pub triggered: bool,
}

impl Queryable<reminders::SqlType, DB> for Reminder<Utc> {
    type Row = (i64, i64, DateTime<Utc>, String, bool);

    fn build(row: Self::Row) -> Self {
        let (_, user_id, reminder_time, reminder_text, triggered) = row;
        Reminder {
            user_id: user_id as u64,
            reminder_time,
            reminder_text,
            triggered,
        }
    }
}

impl<TZ> Insertable<reminders::table> for Reminder<TZ>
where
    TZ: TimeZone,
{
    type Values = (i64, DateTime<Utc>, String, bool);

    fn values(self) -> Self::Values {
        (
            self.user_id as i64,
            self.reminder_time.with_timezone(&Utc),
            self.reminder_text,
            self.triggered,
        )
    }
}
