use std::{fs, io, path::PathBuf, str::FromStr};

use crate::{file_format_ops, simple_ops};

pub enum Command {
    Simple(simple_ops::SimpleOp, String),
    File(file_format_ops::FileOp, PathOrData),
}

// This ensures `Command` remains `Send`. Were we to pass
// `Box<dyn io::BufRead>`, it would not be `Send`.
pub enum PathOrData {
    Path(PathBuf),
    Data(String),
}

impl PathOrData {
    pub fn as_reader(&'_ self) -> anyhow::Result<Box<dyn io::BufRead + '_>> {
        match self {
            Self::Path(path) => {
                let file = fs::File::open(path)?;
                let reader = io::BufReader::new(file);

                Ok(Box::new(reader))
            }
            Self::Data(data) => {
                let reader = io::BufReader::new(data.as_bytes());

                Ok(Box::new(reader))
            }
        }
    }
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.splitn(2, char::is_whitespace);

        let op = parts
            .next()
            .ok_or(anyhow::Error::msg("Missing operation"))?;
        let arg = parts.next().ok_or(anyhow::Error::msg("Missing argument"))?;

        let file_op = file_format_ops::FileOp::from_str(op).ok();
        let simple_op = simple_ops::SimpleOp::from_str(op).ok();

        match (file_op, simple_op) {
            (Some(file_op), _) => Ok(Command::File(file_op, PathOrData::Path(PathBuf::from(arg)))),
            (_, Some(simple_op)) => Ok(Command::Simple(simple_op, arg.to_string())),
            _ => Err(anyhow::Error::msg(format!("Unknown operation: {op}"))),
        }
    }
}

impl Command {
    pub fn exec(self) -> anyhow::Result<String> {
        match self {
            Self::Simple(op, arg) => Ok(op.exec(&arg) + "\n"),
            Self::File(op, input) => op.exec(input.as_reader()?),
        }
    }

    pub fn from_op_name_and_input<R: io::BufRead>(op: &str, mut reader: R) -> anyhow::Result<Self> {
        let file_op = file_format_ops::FileOp::from_str(op).ok();
        let simple_op = simple_ops::SimpleOp::from_str(op).ok();

        let mut input = String::new();
        reader.read_to_string(&mut input)?;

        match (file_op, simple_op) {
            (Some(file_op), _) => Ok(Command::File(file_op, PathOrData::Data(input))),
            (_, Some(simple_op)) => Ok(Command::Simple(simple_op, input)),
            _ => Err(anyhow::Error::msg(format!("Unknown operation: {op}"))),
        }
    }
}
