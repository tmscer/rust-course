// @generated automatically by Diesel CLI.

diesel::table! {
    message (message_id) {
        message_id -> Int8,
        public_id -> Uuid,
        timestamp -> Timestamp,
        user_nickname -> Varchar,
        user_ip -> Varchar,
    }
}

diesel::table! {
    message_file (message_id) {
        message_id -> Int8,
        filename -> Varchar,
        filepath -> Varchar,
        length -> Int8,
        hash -> Varchar,
        mime -> Nullable<Varchar>,
    }
}

diesel::table! {
    message_text (message_id) {
        message_id -> Int8,
        text -> Text,
    }
}

diesel::joinable!(message_file -> message (message_id));
diesel::joinable!(message_text -> message (message_id));

diesel::allow_tables_to_appear_in_same_query!(
    message,
    message_file,
    message_text,
);
