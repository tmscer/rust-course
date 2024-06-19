use common::proto;

use crate::MessageExecutor;

use super::{Client, Server};

impl Server {
    #[tracing::instrument(skip(client_stream, executor), fields(client = %client_stream.peer_addr()?))]
    pub async fn handle_client(
        client_stream: tokio::net::TcpStream,
        executor: &MessageExecutor,
    ) -> anyhow::Result<()> {
        let mut client = Client::new(client_stream);

        while let LoopInstruction::Continue = Self::client_tick(&mut client, executor).await {
            // Continue
        }

        Ok(())
    }

    #[tracing::instrument(skip(client, executor), fields(client = ?client.get_nickname()))]
    async fn client_tick(client: &mut Client, executor: &MessageExecutor) -> LoopInstruction {
        let response = match proto::Payload::read_from(client.get_stream()).await {
            Ok(payload) => match executor.exec(payload.into_inner(), client).await {
                Ok(()) => proto::response::Message::Ok,
                Err(err) => {
                    let msg = err.to_string();
                    proto::response::Message::Err(proto::response::Error::message_exec(msg))
                }
            },
            Err(err) => {
                tracing::debug!("Failed to read message: {err}");
                proto::response::Error::Read(err.to_string()).into()
            }
        };

        let payload = proto::Payload::new(&response);
        if let Err(err) = payload.write_to(&mut client.get_stream()).await {
            tracing::debug!("Failed to send error response: {err}");
            return LoopInstruction::Break;
        }

        LoopInstruction::Continue
    }
}

#[derive(Default)]
enum LoopInstruction {
    #[default]
    Continue,
    Break,
}
