use std::path;

use crate::{receive_streamed_file, Client};

pub struct MessageExecutor {
    root: path::PathBuf,
}

impl MessageExecutor {
    pub fn new(root: path::PathBuf) -> Self {
        Self { root }
    }

    pub async fn exec<S>(
        &self,
        msg: common::proto::request::Message,
        client: &mut Client<S>,
    ) -> anyhow::Result<()>
    where
        S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    {
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
