#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Soft(anyhow::Error),
    #[error("{0}")]
    Hard(anyhow::Error),
}

impl Error {
    pub fn soft<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Soft(error.into())
    }

    pub fn hard<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Hard(error.into())
    }
}
