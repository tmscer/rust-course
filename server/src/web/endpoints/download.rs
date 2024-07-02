use actix_web::{get, web::Bytes};
use uuid::Uuid;

use crate::web::Error;

const FILE_ERROR: &str = "File not found or not accessible";

/// Download message's file, if any.
#[utoipa::path(
    responses(
        (
            status = actix_web::http::StatusCode::OK,
            description = actix_web::http::StatusCode::OK.to_string(),
            headers(
                ("Content-Disposition", description = "Content-Disposition: attachment; filename=\"{filename with exitension}\""),
                ("X-HASH", description = "Content-Disposition: sha256:{hash}"),
            ),
        ),
        (
            status = actix_web::http::StatusCode::NOT_FOUND,
            description = "No file attached to this message or message doesn't exist",
        ),
        (
            status = actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            description = FILE_ERROR,
        )
    ),
    params(
        ("id" = Uuid, description = "Message ID"),
    ),
    operation_id = "download",
)]
#[get("/download/{id}")]
pub async fn handler(
    path: actix_web::web::Path<Uuid>,
    repo: actix_web::web::Data<Box<dyn crate::web::Repository>>,
    args: actix_web::web::Data<crate::ServerArgs>,
) -> Result<impl actix_web::Responder, Error> {
    let id = path.into_inner();

    let message = repo
        .get_message_by_public_id(id)
        .await
        .map_err(Error::internal)?;

    let Some(message) = message else {
        return Ok(actix_web::Either::Left((
            "message doesn't exist",
            actix_web::http::StatusCode::NOT_FOUND,
        )));
    };

    let Some(file) = message.2 else {
        return Ok(actix_web::Either::Left((
            "no file attached to this message",
            actix_web::http::StatusCode::NOT_FOUND,
        )));
    };

    let filepath = args.root.join(file.filepath);
    let fd = tokio::fs::File::open(filepath)
        .await
        .map_err(|_| anyhow::Error::msg(FILE_ERROR))?;

    let stream = stream_file_from_fs(fd);
    let stream = actix_web::body::SizedStream::new(file.length as u64, stream);

    let response = actix_web::HttpResponse::Ok()
        // TODO: Detect mime type
        .content_type("application/octet-stream")
        .insert_header((
            "Content-Disposition",
            format!(r#"attachment; filename="{}""#, file.filename),
        ))
        .insert_header(("X-HASH", format!("sha256:{}", file.hash)))
        .body(stream);

    Ok(actix_web::Either::Right(response))
}

fn stream_file_from_fs(
    file: tokio::fs::File,
) -> impl futures::Stream<Item = Result<Bytes, anyhow::Error>> {
    use tokio::io::AsyncReadExt;

    async_stream::try_stream! {
        let mut buf = [0; 4096];
        let mut reader = tokio::io::BufReader::new(file);

        loop {
            let n = reader.read(&mut buf).await?;

            if n == 0 {
                break;
            }

            yield Bytes::copy_from_slice(&buf[..n]);
        }
    }
}
