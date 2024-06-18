/// Represents a message server sends to client in response to a request.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    /// Request was handled successfully.
    Ok,
    /// Request failed with an error.
    Err(Error),
}

impl From<Result<(), Error>> for Message {
    fn from(result: Result<(), Error>) -> Self {
        match result {
            Ok(()) => Message::Ok,
            Err(err) => Message::Err(err),
        }
    }
}

impl From<Message> for Result<(), Error> {
    fn from(msg: Message) -> Self {
        match msg {
            Message::Ok => Ok(()),
            Message::Err(err) => Err(err),
        }
    }
}

impl From<Error> for Message {
    fn from(err: Error) -> Self {
        Message::Err(err)
    }
}

/// Represents an error that occurred during request handling.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum Error {
    /// Failed to read message - e.g. due to client incompatibility.
    #[error("failed to read message")]
    Read(String),
    #[error("abort understood")]
    ClientAbort,
    /// Failed to execute message - e.g. not enough disk space.
    #[error("message execution error: {0}")]
    MessageExec(String),
    /// Unspecified error.
    #[error("unspecified error: {0}")]
    Unspecified(String),
}

impl Error {
    pub fn message_exec<S: ToString>(msg: S) -> Self {
        Self::MessageExec(msg.to_string())
    }

    pub fn unspecified<S: ToString>(msg: S) -> Self {
        Self::Unspecified(msg.to_string())
    }
}
