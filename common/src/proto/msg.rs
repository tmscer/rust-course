/// Represents a message client sends to server.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    File(String, Vec<u8>),
    Image(String, Vec<u8>),
    Text(String),
}
