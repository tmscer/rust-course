#[async_trait::async_trait]
pub trait Listener: Send {
    type Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send;

    async fn accept_conn(&self) -> anyhow::Result<(Self::Stream, std::net::SocketAddr)>;
}

#[async_trait::async_trait]
impl Listener for tokio::net::TcpListener {
    type Stream = tokio::net::TcpStream;

    async fn accept_conn(&self) -> anyhow::Result<(Self::Stream, std::net::SocketAddr)> {
        self.accept().await.map_err(Into::into)
    }
}
