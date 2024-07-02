use std::num::NonZeroUsize;

use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::{message, message::dsl::*};
use crate::web::FullMessage;

pub struct Repository {
    pool: diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>,
}

impl Repository {
    pub fn new(db_url: &str) -> anyhow::Result<Self> {
        let config = diesel_async::pooled_connection::AsyncDieselConnectionManager::new(db_url);
        let pool = diesel_async::pooled_connection::deadpool::Pool::builder(config).build()?;

        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl crate::web::Repository for Repository {
    async fn get_messages(
        &self,
        username: Option<String>,
        offset: usize,
        limit: NonZeroUsize,
    ) -> anyhow::Result<Vec<(Message, Option<MessageText>, Option<MessageFile>)>> {
        let select = (
            Message::as_select(),
            Option::<MessageText>::as_select(),
            Option::<MessageFile>::as_select(),
        );

        let mut query = message::table
            .left_join(crate::schema::message_text::table)
            .left_join(crate::schema::message_file::table)
            .select(select)
            .order(timestamp.desc())
            .offset(offset.try_into()?)
            .limit(limit.get().try_into()?)
            .into_boxed();

        if let Some(username) = username {
            query = query.filter(user_nickname.eq(username));
        }

        let mut conn = self.pool.get().await?;
        let messages = diesel_async::RunQueryDsl::load(query, &mut conn).await?;

        Ok(messages)
    }

    async fn get_message_by_public_id(
        &self,
        id: uuid::Uuid,
    ) -> anyhow::Result<Option<FullMessage>> {
        let select = (
            Message::as_select(),
            Option::<MessageText>::as_select(),
            Option::<MessageFile>::as_select(),
        );

        let query = message::table
            .left_join(crate::schema::message_text::table)
            .left_join(crate::schema::message_file::table)
            .select(select)
            .filter(public_id.eq(id));

        let mut conn = self.pool.get().await?;

        match diesel_async::RunQueryDsl::first::<FullMessage>(query, &mut conn).await {
            Ok(result) => Ok(Some(result)),
            Err(diesel::NotFound) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete_by_ids(&self, ids: Vec<uuid::Uuid>) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        let query = diesel::delete(message.filter(public_id.eq_any(ids)));
        diesel_async::RunQueryDsl::execute(query, &mut conn).await?;

        Ok(())
    }

    async fn delete_by_username(&self, username: String) -> anyhow::Result<()> {
        let mut conn = self.pool.get().await?;
        let query = diesel::delete(message.filter(user_nickname.eq(username)));
        diesel_async::RunQueryDsl::execute(query, &mut conn).await?;

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::message)]
pub struct NewMessage {
    pub public_id: Uuid,
    pub timestamp: chrono::NaiveDateTime,
    pub user_nickname: String,
    pub user_ip: String,
}

#[derive(Queryable, Selectable, serde::Serialize)]
#[diesel(table_name = crate::schema::message)]
#[allow(unused)]
pub struct Message {
    #[serde(skip)]
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

#[derive(Queryable, Selectable, serde::Serialize)]
#[diesel(table_name = crate::schema::message_file)]
pub struct MessageFile {
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

#[derive(Queryable, Selectable, serde::Serialize)]
#[diesel(table_name = crate::schema::message_text)]
pub struct MessageText {
    pub text: String,
}
