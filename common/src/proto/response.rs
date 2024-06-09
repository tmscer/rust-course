/// Represents a message server sends to client in response to a request.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    Ok,
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

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum Error {
    #[error("failed to read message")]
    Read,
    #[error("message execution error: {0}")]
    MessageExec(String),
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
