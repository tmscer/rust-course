use diesel::prelude::*;
use uuid::Uuid;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message)]
pub struct NewMessage {
    pub public_id: Uuid,
    pub timestamp: chrono::NaiveDateTime,
    pub user_nickname: String,
    pub user_ip: String,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::message)]
#[allow(unused)]
pub struct Message {
    pub message_id: i64,
    pub public_id: Uuid,
    pub timestamp: chrono::NaiveDateTime,
    pub user_nickname: String,
    pub user_ip: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message_file)]
pub struct NewMessageFile {
    pub message_id: i64,
    pub filename: String,
    pub filepath: String,
    pub mime: Option<String>,
    pub length: i64,
    pub hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message_text)]
pub struct NewMessageText {
    pub message_id: i64,
    pub text: String,
}
