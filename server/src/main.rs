use std::{cmp::Ordering, collections::HashMap, net, path};

use clap::Parser;
use common::proto;
use tokio::io::AsyncWriteExt;

#[derive(Parser)]
struct ServerArgs {
    #[clap(short, long, default_value = ".")]
    root: path::PathBuf,
    #[clap(flatten)]
    common: common::cli::Args,
}

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

struct Server {
    listener: tokio::net::TcpListener,
    clients: HashMap<net::SocketAddr, tokio::task::JoinHandle<anyhow::Result<()>>>,
}

#[derive(Debug)]
struct Client {
    stream: tokio::net::TcpStream,
    nickname: Option<String>,
}

impl Client {
    pub fn new(tcp_stream: tokio::net::TcpStream) -> Self {
        Self {
            stream: tcp_stream,
            nickname: None,
        }
    }

    pub fn get_stream(&mut self) -> &mut tokio::net::TcpStream {
        &mut self.stream
    }

    pub fn set_nickname(&mut self, nickname: impl ToString) {
        self.nickname = Some(nickname.to_string());
    }

    pub fn get_nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }
}

#[derive(Default)]
enum LoopInstruction {
    #[default]
    Continue,
    Break,
}

impl Server {
    pub fn new(listener: tokio::net::TcpListener) -> Self {
        Self {
            listener,
            clients: HashMap::new(),
        }
    }

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

    #[tracing::instrument(skip(client_stream, executor), fields(client = %client_stream.peer_addr()?))]
    async fn handle_client(
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

struct MessageExecutor {
    root: path::PathBuf,
}

impl MessageExecutor {
    pub fn new(root: path::PathBuf) -> Self {
        Self { root }
    }

    pub async fn exec(
        &self,
        msg: common::proto::request::Message,
        client: &mut Client,
    ) -> anyhow::Result<()> {
        use common::proto::request::Message;

        tracing::debug!("Handling message");

        let start = tokio::time::Instant::now();

        match msg {
            Message::File(filename, data) => {
                let filepath = self.get_file_path(&filename).await?;
                receive_file(&filepath, &data).await?;
                log_file_receive(start, &filename, data.len() as f64);
            }
            Message::Image(filename, data) => {
                let filepath = self.get_image_path(&filename).await?;
                receive_file(&filepath, &data).await?;
                log_file_receive(start, &filename, data.len() as f64);
            }
            Message::FileStream(filename, size) => {
                let filepath = self.get_file_path(&filename).await?;
                receive_streamed_file(&filepath, size, client.get_stream()).await?;
                log_file_receive(start, &filename, size as f64);
            }
            Message::ImageStream(filename, size) => {
                let filepath = self.get_image_path(&filename).await?;
                receive_streamed_file(&filepath, size, client.get_stream()).await?;
                log_file_receive(start, &filename, size as f64);
            }
            Message::Text(msg) => {
                tracing::info!("Message from: {msg}");
            }
            Message::AnnounceNickname(nickname) => {
                client.set_nickname(&nickname);
                tracing::info!("Client set nickname to {nickname}");
            }
        }

        Ok(())
    }

    async fn get_file_path(&self, filename: &str) -> anyhow::Result<path::PathBuf> {
        let file_root = self.mk_files_dir().await?;
        Ok(file_root.join(filename))
    }

    async fn mk_files_dir(&self) -> anyhow::Result<path::PathBuf> {
        let files_dir = self.root.join("files");
        tokio::fs::create_dir_all(&files_dir).await?;
        Ok(files_dir)
    }

    async fn get_image_path(&self, filename: &str) -> anyhow::Result<path::PathBuf> {
        let image_root = self.mk_images_dir().await?;
        Ok(image_root.join(filename))
    }

    async fn mk_images_dir(&self) -> anyhow::Result<path::PathBuf> {
        let images_dir = self.root.join("images");
        tokio::fs::create_dir_all(&images_dir).await?;
        Ok(images_dir)
    }
}

fn log_file_receive(start: tokio::time::Instant, filename: &str, filesize: f64) {
    let duration = start.elapsed();
    let speed = filesize / duration.as_secs_f64();

    tracing::info!(
        "Received file {filename} ({filesize} bytes) in {duration:?}, speed: {speed}/s",
        filename = filename,
        filesize = human_bytes::human_bytes(filesize),
        speed = human_bytes::human_bytes(speed)
    );
}

async fn receive_file(filepath: &path::Path, data: &[u8]) -> anyhow::Result<()> {
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create(filepath).await?;
    file.write_all(data).await?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum StreamFileError {
    #[error("Expected {expected} bytes but received {received} bytes (too many)")]
    ExpectedMore { expected: u64, received: u64 },
    #[error("Expected {expected} bytes but received {received} bytes (not enough)")]
    ExpectedLess { expected: u64, received: u64 },
    #[error("Client explicitly aborted file transfer without `end` message. Received {received} out of {expected} bytes")]
    Abort { received: u64, expected: u64 },
    #[error("File system error: {0}")]
    Fs(anyhow::Error),
    #[error("Client read error: {0}")]
    Read(anyhow::Error),
}

impl StreamFileError {
    fn fs<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Fs(error.into())
    }

    fn read<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Read(error.into())
    }
}

impl From<StreamFileError> for proto::response::Error {
    fn from(error: StreamFileError) -> Self {
        match error {
            StreamFileError::Fs(e) => Self::MessageExec(e.to_string()),
            StreamFileError::Abort { .. } => Self::ClientAbort,
            // Explicitly listing the errors in case new variants are added.
            // The programmer will have to decide how to handle them instead of
            // automatically converting them to `Read`.
            StreamFileError::Read(_)
            | StreamFileError::ExpectedLess { .. }
            | StreamFileError::ExpectedMore { .. } => Self::Read(error.to_string()),
        }
    }
}

async fn receive_streamed_file(
    filepath: &path::PathBuf,
    expected: u64,
    stream: &mut tokio::net::TcpStream,
) -> Result<(), StreamFileError> {
    let mut file = tokio::fs::File::create(filepath)
        .await
        .map_err(StreamFileError::fs)?;
    let mut received = 0;

    while received <= expected {
        match proto::Payload::read_from(stream)
            .await
            .map(|p| p.into_inner())
        {
            Ok(proto::request::StreamedFile::Payload(data)) => {
                file.write_all(&data).await.map_err(StreamFileError::fs)?;
                received += u64::try_from(data.len()).map_err(StreamFileError::read)?;
            }
            Ok(proto::request::StreamedFile::Abort) => {
                if let Err(e) = tokio::fs::remove_file(filepath)
                    .await
                    .map_err(StreamFileError::fs)
                {
                    tracing::error!("Failed to remove file due to client abort: {e}");
                }

                return Err(StreamFileError::Abort { expected, received });
            }
            Ok(proto::request::StreamedFile::End) => {
                break;
            }
            Err(e) => {
                return Err(StreamFileError::Read(e));
            }
        }
    }

    decide_streamed_file_result(received, expected)
}

fn decide_streamed_file_result(received: u64, expected: u64) -> Result<(), StreamFileError> {
    match expected.cmp(&received) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(StreamFileError::ExpectedMore { expected, received }),
        Ordering::Less => Err(StreamFileError::ExpectedLess { expected, received }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_streamed_file_result_equal() {
        let received = 10;
        let expected = 10;
        let result = decide_streamed_file_result(received, expected);

        assert!(result.is_ok());
    }

    #[test]
    fn test_decide_streamed_file_result_less() {
        let received = 5;
        let expected = 10;
        let result = decide_streamed_file_result(received, expected);

        assert!(matches!(result, Err(StreamFileError::ExpectedMore { .. })));
    }

    #[test]
    fn test_decide_streamed_file_result_more() {
        let received = 15;
        let expected = 10;
        let result = decide_streamed_file_result(received, expected);

        assert!(matches!(result, Err(StreamFileError::ExpectedLess { .. })));
    }
}
