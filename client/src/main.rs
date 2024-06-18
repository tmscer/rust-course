use std::{io, path};

use clap::Parser;

use common::proto;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let args = common::cli::Args::parse();
    let mut conn = tokio::net::TcpStream::connect(args.server_address).await?;

    tracing::info!("Connected to {}", args.server_address);

    let iter_cmds = read_commands(io::stdin().lock());

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

async fn handle_command_should_exit(
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

            tracing::debug!("File size: {file_size}");
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

            tracing::debug!("Image size: {file_size}");
            proto::request::Message::ImageStream(basename, file_size)
        }
        Command::Message(msg) => proto::request::Message::Text(msg),
    };

    let mut bytes_sent = proto::Payload::new(message)
        .write_to(conn)
        .await
        .map_err(Error::hard)?;

    if let Some(filename) = file_to_send {
        bytes_sent += send_stream_file(conn, &filename).await?;
    }

    tracing::debug!("Sent a total of {bytes_sent} bytes");

    Ok(false)
}

async fn send_stream_file(
    conn: &mut tokio::net::TcpStream,
    filepath: &path::Path,
) -> Result<usize, Error> {
    use tokio::io::AsyncReadExt;

    let mut reader = tokio::fs::File::open(&filepath)
        .await
        .map(tokio::io::BufReader::new)
        .map_err(Error::soft)?;

    let mut buf = vec![0; 4096];
    let mut bytes_sent = 0;

    loop {
        let bytes_read = reader.read(&mut buf).await.map_err(Error::soft)?;

        if bytes_read == 0 {
            break;
        }

        let message = proto::request::StreamedFile::Payload(buf[..bytes_read].to_vec());

        bytes_sent += proto::Payload::new(message)
            .write_to(conn)
            .await
            .map_err(Error::hard)?;

        tracing::debug!("Sent chunk of {bytes_sent} bytes");
    }

    proto::Payload::new(proto::request::StreamedFile::End)
        .write_to(conn)
        .await
        .map_err(Error::hard)?;

    Ok(bytes_sent)
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    Soft(anyhow::Error),
    #[error("{0}")]
    Hard(anyhow::Error),
}

impl Error {
    pub fn soft<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Soft(error.into())
    }

    pub fn hard<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Hard(error.into())
    }
}

fn extract_basename(filepath: &path::Path) -> anyhow::Result<String> {
    let basename = filepath
        .file_name()
        .ok_or(anyhow::Error::msg("Failed to extract basename of file"))?
        .to_str()
        .ok_or(anyhow::Error::msg("Failed to decode basename of file"))?;

    Ok(basename.to_string())
}

fn read_commands<R: io::BufRead>(reader: R) -> impl Iterator<Item = anyhow::Result<Command>> {
    let lines = io::BufRead::lines(reader);

    lines.map(|line| Ok(Command::from(line?)))
}

#[derive(Debug)]
pub enum Command {
    File(path::PathBuf),
    Image(path::PathBuf),
    Message(String),
    Quit,
}

impl<T> From<T> for Command
where
    T: AsRef<str>,
{
    fn from(s: T) -> Self {
        let s = s.as_ref();

        if s == ".quit" {
            return Self::Quit;
        }

        if let Some(suffix) = s.strip_prefix(".file ") {
            return Self::File(path::PathBuf::from(suffix));
        }

        if let Some(suffix) = s.strip_prefix(".image ") {
            return Self::Image(path::PathBuf::from(suffix));
        }

        Self::Message(s.to_string())
    }
}
