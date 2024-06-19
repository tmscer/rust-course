use std::path;

use common::proto;

use crate::{send_stream_file, Command, Error};

pub async fn handle_command_should_exit(
    conn: &mut tokio::net::TcpStream,
    cmd: anyhow::Result<Command>,
) -> Result<bool, Error> {
    let cmd = cmd.map_err(Error::hard)?;

    let mut file_to_send = Option::<path::PathBuf>::None;

    let message = match cmd {
        Command::Quit => {
            return Ok(true);
        }
        Command::File(filepath) => {
            let basename = extract_basename(&filepath).map_err(Error::hard)?;
            let metadata = tokio::fs::metadata(&filepath).await.map_err(Error::soft)?;

            if !metadata.is_file() {
                return Err(Error::Soft(anyhow::Error::msg("Only files are supported")));
            }

            let file_size = metadata.len();
            file_to_send = Some(filepath);

            tracing::debug!("File size: {}", human_bytes::human_bytes(file_size as f64));
            proto::request::Message::FileStream(basename, file_size)
        }
        Command::Image(filepath) => {
            let basename = extract_basename(&filepath).map_err(Error::hard)?;
            let metadata = tokio::fs::metadata(&filepath).await.map_err(Error::soft)?;

            if !metadata.is_file() {
                return Err(Error::Soft(anyhow::Error::msg("Only files are supported")));
            } else if !basename.ends_with(".png") {
                return Err(Error::Soft(anyhow::Error::msg(
                    "Only .png images are supported",
                )));
            }

            let file_size = metadata.len();
            file_to_send = Some(filepath);

            tracing::debug!("Image size: {}", human_bytes::human_bytes(file_size as f64));
            proto::request::Message::ImageStream(basename, file_size)
        }
        Command::Message(msg) => proto::request::Message::Text(msg),
        Command::AnnounceNickname(nick) => proto::request::Message::AnnounceNickname(nick),
    };

    let mut bytes_sent = proto::Payload::new(message)
        .write_to(conn)
        .await
        .map_err(Error::hard)?;

    if let Some(filename) = file_to_send {
        bytes_sent += send_stream_file(conn, &filename).await?;
    }

    tracing::debug!(
        "Sent a total of {} bytes",
        human_bytes::human_bytes(bytes_sent as f64)
    );

    Ok(false)
}

fn extract_basename(filepath: &path::Path) -> anyhow::Result<String> {
    let basename = filepath
        .file_name()
        .ok_or(anyhow::Error::msg("Failed to extract basename of file"))?
        .to_str()
        .ok_or(anyhow::Error::msg("Failed to decode basename of file"))?;

    Ok(basename.to_string())
}
