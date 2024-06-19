use clap::Parser;

mod args;
use args::ServerArgs;

mod msg_exec;
pub(crate) use msg_exec::MessageExecutor;

mod receive_file;
pub(crate) use receive_file::receive_streamed_file;

mod server;
pub(crate) use server::{Client, Listener, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let args = ServerArgs::parse();

    let listener = tokio::net::TcpListener::bind(args.common.server_address).await?;
    tracing::info!("Listening on {}", args.common.server_address);

    let server = Server::new(listener);
    let executor = MessageExecutor::new(args.root);

    server.run(executor).await?;

    Ok(())
}
