use std::{collections::HashMap, fs, io::Write, net, path};

use clap::Parser;

#[derive(Parser)]
struct ServerArgs {
    #[clap(short, long, default_value = ".")]
    root: path::PathBuf,
    #[clap(flatten)]
    common: common::cli::Args,
}

fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let args = ServerArgs::parse();

    let listener = net::TcpListener::bind(args.common.server_address)?;
    tracing::info!("Listening on {}", args.common.server_address);

    let server = Server::new(listener);
    let executor = MessageExecutor::new(args.root);

    server.run(executor)?;

    Ok(())
}

struct Server {
    listener: net::TcpListener,
    clients: HashMap<net::SocketAddr, std::thread::JoinHandle<anyhow::Result<()>>>,
}

impl Server {
    pub fn new(listener: net::TcpListener) -> Self {
        Self {
            listener,
            clients: HashMap::new(),
        }
    }

    pub fn run(mut self, executor: MessageExecutor) -> anyhow::Result<()> {
        let executor = std::sync::Arc::new(executor);

        loop {
            self.join_finished_client_threads()?;

            let (client_stream, client_addr) = self.listener.accept()?;

            let executor = executor.clone();
            let handle = std::thread::spawn(move || {
                tracing::info!("Handling connection from {client_addr}");
                tracing::info_span!("handle_client", client = %client_addr)
                    .in_scope(|| Self::handle_client(client_stream, executor.as_ref()))?;
                tracing::info!("Closing connection to {client_addr}");

                Ok(())
            });

            self.clients.insert(client_addr, handle);
        }
    }

    fn handle_client(
        mut client_stream: net::TcpStream,
        executor: &MessageExecutor,
    ) -> anyhow::Result<()> {
        loop {
            match common::proto::Payload::read_from(&mut client_stream) {
                // If we failed to execute a valid message, propagate error further
                Ok(payload) => executor.exec(payload.into_inner())?,
                Err(err) => {
                    // Don't propagate client errors, just stop reading
                    tracing::debug!("Failed to read message: {err}");
                    break;
                }
            }
        }

        Ok(())
    }

    fn join_finished_client_threads(&mut self) -> anyhow::Result<()> {
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
                    .join()
                    .map_err(|_| anyhow::Error::msg("Couldn't join client thread"))??;
            }
        }

        if num_finished > 0 {
            tracing::debug!("Joined {num_finished} client threads");
        }

        Ok(())
    }
}

struct MessageExecutor {
    root: path::PathBuf,
}

impl MessageExecutor {
    pub fn new(root: path::PathBuf) -> Self {
        Self { root }
    }

    pub fn exec(&self, msg: common::proto::Message) -> anyhow::Result<()> {
        use common::proto::Message;

        tracing::debug!("Handling message");

        match msg {
            Message::File(filename, data) => {
                let file_root = self.root.join("files");
                fs::create_dir_all(&file_root)?;
                let filepath = file_root.join(&filename);
                receive_file(&filepath, &data)?;

                tracing::info!(
                    "Received file {filename} ({} bytes) to {:?}",
                    data.len(),
                    filepath
                );
            }
            Message::Image(filename, data) => {
                let image_root = self.root.join("images");
                fs::create_dir_all(&image_root)?;
                let filepath = image_root.join(&filename);
                receive_file(&filepath, &data)?;

                tracing::info!(
                    "Received image {filename} ({} bytes) to {:?}",
                    data.len(),
                    filepath
                );
            }
            Message::Text(msg) => {
                tracing::info!("Message from: {msg}");
            }
        }

        Ok(())
    }
}

fn receive_file(filepath: &path::Path, data: &[u8]) -> anyhow::Result<()> {
    let mut file = fs::File::create(filepath)?;
    file.write_all(data)?;

    Ok(())
}
