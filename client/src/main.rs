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
    let mut conn = tokio::net::TcpStream::connect(args.common.server_address).await?;

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

    Ok(())
}

fn read_commands<R: io::BufRead>(reader: R) -> impl Iterator<Item = anyhow::Result<Command>> {
    let lines = io::BufRead::lines(reader);

    lines.map(|line| Ok(Command::from(line?)))
}
