use super::DB;
use crate::schema::reminders;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use diesel::{ExpressionMethods, Insertable, Queryable};
use serenity::client::Context as SContext;
use serenity::model::channel::{Channel, Message};
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub struct Reminder<TZ>
where
    TZ: TimeZone,
{
    reminder_id: Option<i64>,
    pub user_id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub reminder_time: DateTime<TZ>,
    pub reminder_text: String,
    pub triggered: bool,
}

impl<TZ: TimeZone> Reminder<TZ> {
    pub fn new(
        user_id: u64,
        guild_id: u64,
        channel_id: u64,
        reminder_time: DateTime<TZ>,
        reminder_text: String,
    ) -> Self {
        Self {
            reminder_id: None,
            user_id,
            guild_id,
            channel_id,
            reminder_time,
            reminder_text,
            triggered: false,
        }
    }

    pub fn from_message(
        msg: &Message,
        reminder_time: DateTime<TZ>,
        reminder_text: String,
    ) -> Option<Self> {
        let guild_id = msg.guild_id?.0;
        Some(Self::new(
            msg.author.id.0,
            guild_id,
            msg.channel_id.0,
            reminder_time,
            reminder_text,
        ))
    }

    pub async fn trigger(&mut self, ctx: &SContext) -> Result<bool> {
        let now = Utc::now();

        if now < self.reminder_time {
            return Result::<_>::Ok(false).with_context(|| {
                anyhow!(
                    "Too early for reminder {:?} to trigger (now: {}, reminder time: {})",
                    self.reminder_id,
                    now,
                    self.reminder_time.with_timezone(&Utc),
                )
            });
        }

        let channel = ctx
            .http
            .get_channel(self.channel_id)
            .await
            .with_context(|| {
                anyhow!(
                    "Could not find channel {} for reminder {:?}",
                    self.channel_id,
                    self.reminder_id
                )
            })?;

        if let Channel::Guild(gc) = channel {
            let member = ctx
                .http
                .get_member(self.guild_id, self.user_id)
                .await
                .with_context(|| {
                    anyhow!(
                        "Could not find member {} in guild {} for reminder {:?}",
                        self.user_id,
                        self.guild_id,
                        self.reminder_id
                    )
                })?;

            let content = format!("Reminding {}: {}", member, self.reminder_text);

            gc.say(ctx, content).await.with_context(|| {
                anyhow!(
                    "Failed to send reminder message for reminder {:?} in guild {} channel {}",
                    self.reminder_id,
                    self.guild_id,
                    self.channel_id
                )
            })?;

            Ok(true)
        } else {
            Err(anyhow!(
                "Could not send reminder {:?} because channel {} is not a guild channel",
                self.reminder_id,
                self.channel_id
            ))
        }
    }
}

impl Queryable<reminders::SqlType, DB> for Reminder<Utc> {
    type Row = (i64, i64, i64, i64, DateTime<Utc>, String, bool);

    fn build(row: Self::Row) -> Self {
        let (reminder_id, user_id, guild_id, channel_id, reminder_time, reminder_text, triggered) =
            row;
        let user_id = user_id as u64;
        let guild_id = guild_id as u64;
        let channel_id = channel_id as u64;
        Reminder {
            reminder_id: Some(reminder_id),
            user_id,
            guild_id,
            channel_id,
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
    type Values = <(
        diesel::dsl::Eq<reminders::user_id, i64>,
        diesel::dsl::Eq<reminders::guild_id, i64>,
        diesel::dsl::Eq<reminders::channel_id, i64>,
        diesel::dsl::Eq<reminders::reminder_time, DateTime<Utc>>,
        diesel::dsl::Eq<reminders::reminder_text, String>,
        diesel::dsl::Eq<reminders::triggered, bool>,
    ) as Insertable<reminders::table>>::Values;

    fn values(self) -> Self::Values {
        (
            reminders::user_id.eq(self.user_id as i64),
            reminders::guild_id.eq(self.guild_id as i64),
            reminders::channel_id.eq(self.channel_id as i64),
            reminders::reminder_time.eq(self.reminder_time.with_timezone(&Utc)),
            reminders::reminder_text.eq(self.reminder_text),
            reminders::triggered.eq(self.triggered),
        )
            .values()
    }
}
