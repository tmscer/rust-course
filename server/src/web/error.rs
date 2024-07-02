#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("internal error occurred: {0}")]
    InternalError(#[from] anyhow::Error),
}

impl Error {
    pub fn internal<E>(error: E) -> Self
    where
        E: Into<anyhow::Error>,
    {
        Self::InternalError(error.into())
    }
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match &self {
            Self::InternalError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::build(self.status_code()).body(self.to_string())
    }
}
