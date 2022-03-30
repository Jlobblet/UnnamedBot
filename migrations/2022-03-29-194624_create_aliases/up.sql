CREATE TABLE aliases (
    alias_id bigserial PRIMARY KEY
,   user_id bigint references users(user_id) NOT NULL
,   command_name text NOT NULL
,   command_text text NOT NULL
)
