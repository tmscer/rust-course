use std::{
    fs,
    io::{self, Write},
    net, path,
};

fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let server_addr = common::get_server_addr(std::env::args().nth(1).as_deref())?;
    let mut conn = net::TcpStream::connect(server_addr)?;

    tracing::info!("Connected to {server_addr}");

    let iter_cmds = read_commands(io::stdin().lock());
    let iter_cmd_results = iter_cmds.map(|cmd| handle_command_should_exit(&mut conn, cmd));

    for cmd_result in iter_cmd_results {
        match cmd_result {
            Ok(true) => {
                tracing::info!("Exiting...");
                break;
            }
            Ok(false) => {
                tracing::info!("Command execution finished");
            }
            Err(Error::Soft(err)) => {
                tracing::warn!("Non-fatal error: {err}");
            }
            Err(Error::Hard(err)) => {
                tracing::error!("Exiting due to: {err}");
                return Err(err);
            }
        }
    }

    Ok(())
}

fn handle_command_should_exit(
    conn: &mut net::TcpStream,
    cmd: anyhow::Result<Command>,
) -> Result<bool, Error> {
    let cmd = cmd.map_err(Error::hard)?;

    let message = match cmd {
        Command::Quit => {
            return Ok(true);
        }
        Command::File(filepath) => {
            let basename = extract_basename(&filepath).map_err(Error::hard)?;
            let contents = fs::read(filepath).map_err(Error::soft)?;

            common::Message::File(basename, contents)
        }
        Command::Image(filepath) => {
            let basename = extract_basename(&filepath).map_err(Error::hard)?;

            if !basename.ends_with(".png") {
                return Err(Error::Soft(anyhow::Error::msg(
                    "Only .png images are supported",
                )));
            }

            let contents = fs::read(filepath).map_err(Error::soft)?;

            common::Message::File(basename, contents)
        }
        Command::Message(msg) => common::Message::Text(msg),
    };

    let payload = serde_cbor::to_vec(&message).map_err(Error::hard)?;
    let len: common::Len = payload.len().try_into().map_err(Error::hard)?;

    conn.write_all(&len.to_be_bytes()).map_err(Error::hard)?;
    conn.write_all(&payload).map_err(Error::hard)?;

    let bytes_sent = std::mem::size_of::<common::Len>() + payload.len();
    tracing::debug!("Sent {bytes_sent} bytes");

    Ok(false)
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
