use std::io;

use clap::Parser;

use common::proto;

mod args;
use args::ClientArgs;

mod command;
pub(crate) use command::Command;

mod error;
pub(crate) use error::Error;

mod send_file;
pub(crate) use send_file::send_stream_file;

mod send_command;
pub(crate) use send_command::handle_command_should_exit;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let args = ClientArgs::parse();
    let mut conn = create_connection(&args).await?;

    tracing::info!("Connected to {}", args.common.server_address);

    let announce_nick_cmd = Command::AnnounceNickname(args.nickname);
    // For some reason using `anyhow::Result::Ok(announce_nick_cmd)` doesn't work - Rust cannot infer the error type E.
    let iter_cmds = std::iter::once(Result::<_, anyhow::Error>::Ok(announce_nick_cmd));
    let iter_cmds = iter_cmds.chain(read_commands(io::stdin().lock()));

    for cmd in iter_cmds {
        match handle_command_should_exit(&mut conn, cmd).await {
            Ok(true) => {
                tracing::info!("Exiting...");
                break;
            }
            Ok(false) => {
                tracing::info!("Command sent");
            }
            Err(Error::Soft(err)) => {
                tracing::warn!("Non-fatal error: {err}");
                continue;
            }
            Err(Error::Hard(err)) => {
                tracing::error!("Exiting due to: {err}");
                return Err(err);
            }
        }

        tracing::info!("Waiting for response...");
        let response = proto::Payload::<proto::response::Message>::read_from(&mut conn)
            .await
            .map_err(Error::hard)?
            .into_inner();

        match response {
            proto::response::Message::Ok => {
                tracing::info!("Request was successful");
            }
            proto::response::Message::Err(error) => {
                tracing::error!("Server responded with an error: {error}");
            }
        }
    }

    use tokio::io::AsyncWriteExt;
    conn.shutdown().await?;

    Ok(())
}

fn read_commands<R: io::BufRead>(reader: R) -> impl Iterator<Item = anyhow::Result<Command>> {
    let lines = io::BufRead::lines(reader);

    lines.map(|line| Ok(Command::from(line?)))
}

async fn create_connection(
    args: &ClientArgs,
) -> anyhow::Result<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin> {
    let conn = tokio::net::TcpStream::connect(args.common.server_address).await?;

    #[cfg(feature = "mtls")]
    let conn = {
        let connector = create_connector(args)?;
        let domain = rustls_pki_types::ServerName::try_from(args.mtls.cert_domain.clone())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid dnsname"))?
            .to_owned();

        connector.connect(domain, conn).await?
    };

    Ok(conn)
}

#[cfg(feature = "mtls")]
fn create_connector(args: &ClientArgs) -> anyhow::Result<tokio_rustls::TlsConnector> {
    let certs = common::tls::load_certs(&args.mtls.cert)?;
    let priv_key = common::tls::load_keys(&args.mtls.key)?;
    let roots_store = common::tls::load_root_certs(&args.mtls.ca_cert)?;

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots_store)
        .with_client_auth_cert(certs, priv_key)?;
    let connector = tokio_rustls::TlsConnector::from(std::sync::Arc::new(config));

    Ok(connector)
}
