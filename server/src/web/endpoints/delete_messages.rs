use actix_web::post;
use uuid::Uuid;

use super::{render_table, SearchParams};
use crate::web::Error;

/// Delete messages.
///
/// File contents will be deleted by a cron job. In case of accidents, contact the administrator.
///
/// The method would be delete but `<form>` only supports GET and POST.
#[utoipa::path(
    responses(
        (
            status = actix_web::http::StatusCode::OK,
            description = "Message has been deleted. Returning HTML table.",
        ),
    ),
)]
#[tracing::instrument(skip(repo))]
#[post("/delete")]
pub async fn handler(
    params: actix_web::web::Form<DeleteParams>,
    repo: actix_web::web::Data<Box<dyn crate::web::Repository>>,
    args: actix_web::web::Data<crate::ServerArgs>,
) -> Result<impl actix_web::Responder, Error> {
    match params.into_inner() {
        DeleteParams::Specific { id } => {
            repo.delete_by_ids(vec![id])
                .await
                .map_err(Error::internal)?;
        }
        DeleteParams::User { username } => {
            repo.delete_by_username(username)
                .await
                .map_err(Error::internal)?;
        }
    }

    render_table(
        repo.as_ref().as_ref(),
        SearchParams::default(),
        args.as_ref(),
    )
    .await
    .map_err(Error::internal)
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(untagged)]
pub enum DeleteParams {
    Specific {
        /// Delete this specific message.
        id: Uuid,
    },
    User {
        /// Delete all messages from this user.
        username: String,
    },
}
