use actix_web::{get, Responder};
use prometheus::Encoder;

use crate::web::Error;

/// Get Prometheus metrics in text format.
///
/// Uses <https://docs.rs/prometheus/0.13.4/prometheus/struct.TextEncoder.html>.
///
/// # Metrics Description
///
/// - `http_requests_total`: Total number of HTTP requests over server's lifetime. Excludes metrics and docs requests.
/// - `messages_total`: Total number of messages handled by server. File streaming messages are counted as one.
/// - `messages_received_bytes`: Total number of bytes received when handling messages.
/// - `messages_sent_bytes`: Total number of bytes sent when handling messages.
/// - `active_connections`: Number of active connections to the server.
#[utoipa::path(
    responses(
        (
            status = actix_web::http::StatusCode::OK,
            description = "Metrics in text format.",
        ),
        (
            status = actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            description = "Error while encoding metrics",
        )
    ),
    operation_id = "get_metrics",

)]
#[tracing::instrument]
#[get("/metrics")]
pub async fn handler() -> Result<impl actix_web::Responder, Error> {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();

    let encoded = encoder
        .encode_to_string(&metric_families)
        .map_err(Error::internal)?;

    Ok(encoded
        .customize()
        .insert_header((actix_web::http::header::CONTENT_TYPE, encoder.format_type())))
}
