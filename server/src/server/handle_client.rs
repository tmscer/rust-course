use common::proto;

use crate::{Client, MessageExecutor};

use super::Server;

impl<L> Server<L>
where
    L: Send,
{
    #[tracing::instrument(skip(client, executor), fields(client = %client.get_address()))]
    pub async fn handle_client<S>(
        mut client: Client<S>,
        executor: &MessageExecutor,
    ) -> anyhow::Result<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        while let LoopInstruction::Continue = Self::client_tick(&mut client, executor).await {
            // Continue
        }

        Ok(())
    }

    #[tracing::instrument(skip(client, executor), fields(client = ?client.get_nickname()))]
    async fn client_tick<S>(client: &mut Client<S>, executor: &MessageExecutor) -> LoopInstruction
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
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
