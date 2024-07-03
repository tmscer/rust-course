lazy_static::lazy_static! {
    pub static ref HTTP_REQUESTS_TOTAL: prometheus::IntCounter = prometheus::IntCounter::with_opts(
        prometheus::Opts::new(
            "http_requests_total",
            "Total number of HTTP requests over server's lifetime. Excludes metrics and docs requests.",
        )
    ).expect("a metric");
    pub static ref MESSAGES_TOTAL: prometheus::IntCounter = prometheus::IntCounter::with_opts(
        prometheus::Opts::new(
            "messages_total",
            "Total number of messages handled by server. File streaming messages are counted as one.",
        )
    ).expect("a metric");
}

pub fn register(registry: &prometheus::Registry) -> Result<(), prometheus::Error> {
    registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    registry.register(Box::new(MESSAGES_TOTAL.clone()))?;

    Ok(())
}
