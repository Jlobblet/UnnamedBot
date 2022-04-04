table! {
    aliases (alias_id) {
        alias_id -> Int8,
        user_id -> Int8,
        guild_id -> Int8,
        command_name -> Text,
        command_text -> Text,
    }
}

table! {
    reminders (reminder_id) {
        reminder_id -> Int8,
        user_id -> Int8,
        reminder_time -> Timestamptz,
        reminder_text -> Text,
        triggered -> Bool,
    }
}

table! {
    users (user_id) {
        user_id -> Int8,
        timezone -> Nullable<Text>,
    }
}

joinable!(aliases -> users (user_id));
joinable!(reminders -> users (user_id));

allow_tables_to_appear_in_same_query!(aliases, reminders, users,);
