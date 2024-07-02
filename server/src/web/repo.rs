use std::num::NonZeroUsize;

use crate::db::{Message, MessageFile, MessageText};

pub type FullMessage = (Message, Option<MessageText>, Option<MessageFile>);

#[async_trait::async_trait]
pub trait Repository: Sync + Send + 'static {
    async fn get_messages(
        &self,
        username: Option<String>,
        offset: usize,
        limit: NonZeroUsize,
    ) -> anyhow::Result<Vec<FullMessage>>;

    async fn get_message_by_public_id(
        &self,
        public_id: uuid::Uuid,
    ) -> anyhow::Result<Option<FullMessage>>;

    async fn delete_by_ids(&self, ids: Vec<uuid::Uuid>) -> anyhow::Result<()>;

    async fn delete_by_username(&self, username: String) -> anyhow::Result<()>;
}

// #[derive(serde::Serialize)]
// pub struct Message {
//     #[serde(flatten)]
//     pub msg: crate::db::Message,
//     #[serde(flatten)]
//     pub subtype: MessageSubtype,
// }

// #[derive(serde::Serialize)]
// pub enum MessageSubtype {
//     Text(crate::db::NewMessageText),
//     File(crate::db::NewMessageFile),
// }
