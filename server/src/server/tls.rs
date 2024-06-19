use std::sync::Arc;

use super::Listener;

use tokio_rustls::{rustls, TlsAcceptor};

pub struct TlsListener<L> {
    listener: L,
    acceptor: tokio_rustls::TlsAcceptor,
}

impl<L> TlsListener<L> {
    pub fn new(listener: L, config: rustls::ServerConfig) -> Self {
        let acceptor = TlsAcceptor::from(Arc::new(config));

        Self { listener, acceptor }
    }
}

#[async_trait::async_trait]
impl<L> Listener for TlsListener<L>
where
    L: Listener,
{
    type Stream = tokio_rustls::server::TlsStream<L::Stream>;

    async fn accept_conn(&self) -> anyhow::Result<(Self::Stream, std::net::SocketAddr)> {
        let (stream, addr) = self.listener.accept_conn().await?;
        let stream = self.acceptor.accept(stream).await?;

        Ok((stream, addr))
    }
}
