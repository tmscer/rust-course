use actix_web::get;

use super::{render_table, SearchParams};
use crate::web::Error;

/// Get messages processed by the server.
#[utoipa::path(
    responses(
        (
            status = actix_web::http::StatusCode::OK,
            description = actix_web::http::StatusCode::OK.to_string(),
            content_type = actix_web::http::header::ContentType::html(),
        ),
    ),
    params(
        SearchParams,
    ),
    operation_id = "get_messages",
)]
#[tracing::instrument(skip(repo))]
#[get("/")]
pub async fn handler(
    query: actix_web::web::Query<SearchParams>,
    repo: actix_web::web::Data<Box<dyn crate::web::Repository>>,
    args: actix_web::web::Data<crate::ServerArgs>,
) -> Result<impl actix_web::Responder, Error> {
    render_table(repo.as_ref().as_ref(), query.into_inner(), args.as_ref())
        .await
        .map_err(Error::internal)
}
