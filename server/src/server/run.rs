use crate::{MessageExecutor, Server};

impl Server {
    pub async fn run(mut self, executor: MessageExecutor) -> anyhow::Result<()> {
        let executor = std::sync::Arc::new(executor);

        loop {
            self.join_finished_clients().await?;

            let (client_stream, client_addr) = self.listener.accept().await?;

            let executor = executor.clone();

            let handle = tokio::spawn(async move {
                tracing::info!("Handling connection from {client_addr}");
                Self::handle_client(client_stream, executor.as_ref()).await?;
                tracing::info!("Closing connection to {client_addr}");

                anyhow::Ok(())
            });

            self.clients.insert(client_addr, handle);
        }
    }

    async fn join_finished_clients(&mut self) -> anyhow::Result<()> {
        let mut finished_clients = vec![];
        for (client_addr, handle) in self.clients.iter_mut() {
            if handle.is_finished() {
                finished_clients.push(*client_addr);
            }
        }

        let num_finished = finished_clients.len();

        for finished_client_addr in finished_clients {
            if let Some(handle) = self.clients.remove(&finished_client_addr) {
                handle
                    .await
                    .map_err(|_| anyhow::Error::msg("Couldn't join client thread"))??;
            }
        }

        if num_finished > 0 {
            tracing::debug!("Joined {num_finished} client threads");
        }

        Ok(())
    }
}
