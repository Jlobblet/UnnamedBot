CREATE TABLE reminders (
    reminder_id bigserial PRIMARY KEY
,   user_id bigint references users(user_id) NOT NULL
,   guild_id bigint NOT NULL
,   channel_id bigint NOT NULL
,   reminder_time timestamp with time zone NOT NULL
,   reminder_text text NOT NULL
,   triggered bool NOT NULL DEFAULT false
);
