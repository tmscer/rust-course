/// Represents a message client sends to server.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    /// Filename and file data.
    File(String, Vec<u8>),
    /// Filename and how many bytes will be sent as file data.
    /// Filesize is represented as [`u64`] instead of [`usize`] to make it platform-independent.
    FileStream(String, u64),
    /// Filename and image data.
    Image(String, Vec<u8>),
    /// Filename and how many bytes will be sent as image data.
    /// Filesize is represented as [`u64`] instead of [`usize`] to make it platform-independent.
    ImageStream(String, u64),
    /// Text message.
    Text(String),
}

/// Represents a message client sends to server while streaming a file or image to it.
/// Data is sent in chunks and the client can choose to quit anytime and use the connection
/// for something else.
///
/// Since the protocol requires first sending the message size, as mandated by [`crate::proto::Payload`],
/// servers can reject messages that are too long.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum StreamedFile {
    /// File data chunk.
    Payload(Vec<u8>),
    /// Abort the current file transfer.
    Abort,
    /// End of the current file transfer - the whole file has been sent.
    End,
}
