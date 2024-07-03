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
    pub static ref MESSAGES_RECEIVED_BYTES: prometheus::IntCounter = prometheus::IntCounter::with_opts(
        prometheus::Opts::new(
            "messages_received_bytes",
            "Total number of bytes received when handling messages.",
        )
    ).expect("a metric");
    pub static ref MESSAGES_SENT_BYTES: prometheus::IntCounter = prometheus::IntCounter::with_opts(
        prometheus::Opts::new(
            "messages_sent_bytes",
            "Total number of bytes sent when handling messages.",
        )
    ).expect("a metric");
}

pub fn register(registry: &prometheus::Registry) -> Result<(), prometheus::Error> {
    registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone()))?;
    registry.register(Box::new(MESSAGES_TOTAL.clone()))?;
    registry.register(Box::new(MESSAGES_RECEIVED_BYTES.clone()))?;
    registry.register(Box::new(MESSAGES_SENT_BYTES.clone()))?;

    Ok(())
}

/// A wrapper around a listener that meters the amount of data read and written on each accepted stream.
pub struct MeteredListener<L> {
    inner: L,
    read_metric: Option<prometheus::IntCounter>,
    write_metric: Option<prometheus::IntCounter>,
}

impl<L> MeteredListener<L> {
    pub fn new(inner: L) -> Self {
        Self {
            inner,
            read_metric: None,
            write_metric: None,
        }
    }

    pub fn set_read_metric(&mut self, metric: prometheus::IntCounter) {
        self.read_metric = Some(metric);
    }

    pub fn set_write_metric(&mut self, metric: prometheus::IntCounter) {
        self.write_metric = Some(metric);
    }
}

#[async_trait::async_trait]
impl<L> crate::server::Listener for MeteredListener<L>
where
    L: crate::server::Listener,
    Self: Send + Sync,
{
    type Stream = MeteredStream<L::Stream>;

    async fn accept_conn(&self) -> anyhow::Result<(Self::Stream, std::net::SocketAddr)> {
        self.inner.accept_conn().await.map(|(stream, addr)| {
            let mut stream = MeteredStream::new(stream);

            if let Some(metric) = self.read_metric.clone() {
                stream.set_read_metric(metric);
            }

            if let Some(metric) = self.write_metric.clone() {
                stream.set_write_metric(metric);
            }

            (stream, addr)
        })
    }
}

/// A wrapper around a stream that meters the amount of data read and written.
#[pin_project::pin_project]
pub struct MeteredStream<T> {
    #[pin]
    stream: T,
    read_metric: Option<prometheus::IntCounter>,
    write_metric: Option<prometheus::IntCounter>,
}

impl<T> MeteredStream<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            read_metric: None,
            write_metric: None,
        }
    }

    pub fn set_read_metric(&mut self, metric: prometheus::IntCounter) {
        self.read_metric = Some(metric);
    }

    pub fn set_write_metric(&mut self, metric: prometheus::IntCounter) {
        self.write_metric = Some(metric);
    }
}

impl<T> tokio::io::AsyncRead for MeteredStream<T>
where
    T: tokio::io::AsyncRead,
{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let project = self.project();
        let poll = project.stream.poll_read(cx, buf);

        if let std::task::Poll::Ready(Ok(())) = poll {
            if let Some(metric) = project.read_metric {
                metric.inc_by(buf.filled().len() as u64);
            }
        }

        poll
    }
}

// implement all methods as not to introduce default impls even
// though `T` might implement them
impl<T> tokio::io::AsyncWrite for MeteredStream<T>
where
    T: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let project = self.project();
        let poll = project.stream.poll_write(cx, buf);

        if let std::task::Poll::Ready(Ok(bytes)) = poll {
            if let Some(metric) = project.write_metric {
                metric.inc_by(bytes as u64);
            }
        }

        poll
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().stream.poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.project().stream.poll_shutdown(cx)
    }

    fn is_write_vectored(&self) -> bool {
        self.stream.is_write_vectored()
    }

    fn poll_write_vectored(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let project = self.project();
        let poll = project.stream.poll_write_vectored(cx, bufs);

        if let std::task::Poll::Ready(Ok(bytes)) = poll {
            if let Some(metric) = project.write_metric {
                metric.inc_by(bytes as u64);
            }
        }

        poll
    }
}
