use clap::Parser;

mod args;
use args::ServerArgs;

mod msg_exec;
pub(crate) use msg_exec::MessageExecutor;

mod receive_file;
pub(crate) use receive_file::receive_streamed_file;

mod server;
#[cfg(feature = "mtls")]
pub(crate) use server::TlsListener;
pub(crate) use server::{Client, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let args = ServerArgs::parse();
    let listener = get_listener(&args).await?;

    tracing::info!("Listening on {}", args.common.server_address);

    let server = Server::new(listener);
    let executor = MessageExecutor::new(args.root);

    server.run(executor).await?;

    Ok(())
}

async fn get_listener(args: &ServerArgs) -> anyhow::Result<impl server::Listener> {
    let listener = tokio::net::TcpListener::bind(args.common.server_address).await?;

    #[cfg(feature = "mtls")]
    let listener = {
        let certs = common::tls::load_certs(&args.mtls.cert)?;
        let priv_key = common::tls::load_keys(&args.mtls.key)?;
        let roots_store = common::tls::load_root_certs(&args.mtls.ca_cert)?;

        let verifier = rustls::server::WebPkiClientVerifier::builder(roots_store.into()).build()?;

        let tls_config = rustls::ServerConfig::builder()
            .with_client_cert_verifier(verifier)
            .with_single_cert(certs, priv_key)?;

        TlsListener::new(listener, tls_config)
    };

    Ok(listener)
}
