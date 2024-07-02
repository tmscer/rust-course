use std::path;

use crate::{receive_streamed_file, Client};

pub struct MessageExecutor {
    root: path::PathBuf,
    on_execute: Option<tokio::sync::mpsc::Sender<ExecNotification>>,
}

#[derive(Debug)]
pub struct ExecNotification {
    pub client_nickname: Option<String>,
    pub client_ip: std::net::IpAddr,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: Message,
}

type Hash = sha2::Sha256;

/// Shorted representation of [`common::proto::request::Message`] for notification purposes.
#[derive(Debug)]
pub enum Message {
    /// Text message and its content.
    Text(String),
    /// File (and images) message, its filename and sha256 hash.
    File {
        filename: String,
        filepath: String,
        mime: Option<String>,
        hash: Vec<u8>,
        length: u64,
    },
}

impl MessageExecutor {
    pub fn new(root: path::PathBuf) -> Self {
        Self {
            root,
            on_execute: None,
        }
    }

    pub fn with_notifications(
        mut self,
        on_execute: tokio::sync::mpsc::Sender<ExecNotification>,
    ) -> Self {
        self.on_execute = Some(on_execute);
        self
    }

    pub async fn exec<S>(
        &self,
        msg: common::proto::request::Message,
        client: &mut Client<S>,
    ) -> anyhow::Result<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
        use common::proto::request;

        tracing::debug!("Handling message");

        let start = tokio::time::Instant::now();

        let notification = match msg {
            request::Message::File(filename, data) => {
                let filepath = self.get_file_path(&filename).await?;
                let info = receive_file::<Hash>(&filepath, &data).await?;
                log_file_receive(start, &filename, data.len() as f64);

                Some(Message::File {
                    filename,
                    filepath: filepath.to_str().unwrap_or_default().to_string(),
                    mime: info.mime,
                    hash: info.hash,
                    length: info.length,
                })
            }
            request::Message::Image(filename, data) => {
                let filepath = self.get_image_path(&filename).await?;
                let info = receive_file::<Hash>(&filepath, &data).await?;
                log_file_receive(start, &filename, data.len() as f64);

                Some(Message::File {
                    filename,
                    filepath: filepath.to_str().unwrap_or_default().to_string(),
                    mime: info.mime,
                    hash: info.hash,
                    length: info.length,
                })
            }
            request::Message::FileStream(filename, size) => {
                let filepath = self.get_file_path(&filename).await?;
                let info =
                    receive_streamed_file::<Hash, _>(&filepath, size, client.get_stream()).await?;
                log_file_receive(start, &filename, size as f64);

                Some(Message::File {
                    filename,
                    filepath: filepath.to_str().unwrap_or_default().to_string(),
                    mime: info.mime,
                    hash: info.hash,
                    length: info.length,
                })
            }
            request::Message::ImageStream(filename, size) => {
                let filepath = self.get_image_path(&filename).await?;
                let info =
                    receive_streamed_file::<Hash, _>(&filepath, size, client.get_stream()).await?;
                log_file_receive(start, &filename, size as f64);

                Some(Message::File {
                    filename,
                    filepath: filepath.to_str().unwrap_or_default().to_string(),
                    mime: info.mime,
                    hash: info.hash,
                    length: info.length,
                })
            }
            request::Message::Text(msg) => {
                tracing::info!("Message from: {msg}");

                Some(Message::Text(msg))
            }
            request::Message::AnnounceNickname(nickname) => {
                client.set_nickname(&nickname);
                tracing::info!("Client set nickname to {nickname}");

                None
            }
        };

        if let Some((notification, sender)) = notification.zip(self.on_execute.as_ref()) {
            let notification = ExecNotification {
                client_nickname: client.get_nickname().map(ToString::to_string),
                client_ip: client.get_address().ip(),
                timestamp: chrono::Utc::now(),
                message: notification,
            };

            sender.send(notification).await?;
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

pub struct StreamInfo {
    pub length: u64,
    pub hash: Vec<u8>,
    pub mime: Option<String>,
}

async fn receive_file<H: sha2::Digest>(
    filepath: &path::Path,
    data: &[u8],
) -> anyhow::Result<StreamInfo> {
    use tokio::io::AsyncWriteExt;

    let mut file = tokio::fs::File::create(filepath).await?;
    file.write_all(data).await?;

    let mut hasher = H::new();
    hasher.update(data);

    let hash = hasher.finalize().to_vec();
    let info = StreamInfo {
        length: data.len() as u64,
        hash,
        mime: Some(tree_magic_mini::from_u8(data).to_string()),
    };

    Ok(info)
}
