use actix_web::Responder;
use utoipa::OpenApi;

use super::endpoints;

use endpoints::delete_messages::DeleteParams;

pub fn endpoints(prefix: &str) -> impl actix_web::dev::HttpServiceFactory + 'static {
    actix_web::web::scope(prefix)
        .service(spec_redoc("/redoc"))
        .service(spec_scalar("/scalar"))
        .service(spec_json("/openapi.json"))
}

fn spec_redoc(prefix: &str) -> impl actix_web::dev::HttpServiceFactory + 'static {
    use utoipa_redoc::Servable;
    utoipa_redoc::Redoc::with_url(prefix.to_owned(), ApiDoc::openapi())
}

fn spec_scalar(prefix: &str) -> impl actix_web::dev::HttpServiceFactory + 'static {
    use utoipa_scalar::Servable;
    utoipa_scalar::Scalar::with_url(prefix.to_owned(), ApiDoc::openapi())
}

fn spec_json(prefix: &str) -> impl actix_web::dev::HttpServiceFactory + 'static {
    actix_web::web::scope(prefix).route("", actix_web::web::get().to(spec_json_handler))
}

async fn spec_json_handler() -> impl actix_web::Responder {
    ApiDoc::openapi().to_json().customize().insert_header((
        actix_web::http::header::CONTENT_TYPE,
        actix_web::http::header::ContentType::json(),
    ))
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "API Documentation",
        description = r#"This API doc is available in three formats:<br>
        <ul>
            <li><a href="./redoc">Redoc</a></li>
            <li><a href="./scalar">Scalar</a></li>
            <li><a href="./openapi.json">JSON</a> (if you're extra hardcore)</li>
        </ul>
        It describes web server of this Rust course application which is for viewing and deleting messages
        handled by the non-HTTP server.
        "#,
    ),
    external_docs(
        url = "https://github.com/tmscer/rust-course",
        description = "GitHub Repository"
    ),
    paths(
        endpoints::get_messages::handler,
        endpoints::delete_messages::handler,
        endpoints::download::handler,
    ),
    components(schemas(DeleteParams))
)]
pub struct ApiDoc;
