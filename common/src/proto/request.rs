/// Represents a message client sends to server.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    /// Filename and file data.
    File(String, Vec<u8>),
    /// Filename and image data.
    Image(String, Vec<u8>),
    /// Text message.
    Text(String),
}
