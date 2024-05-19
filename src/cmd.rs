use std::{path::PathBuf, str::FromStr};

use crate::{file_format_ops, simple_ops};

pub enum Command {
    Simple(simple_ops::SimpleOp, String),
    File(file_format_ops::FileOp, PathBuf),
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
            (Some(file_op), _) => Ok(Command::File(file_op, PathBuf::from(arg))),
            (_, Some(simple_op)) => Ok(Command::Simple(simple_op, arg.to_string())),
            _ => Err(anyhow::Error::msg(format!("Unknown operation: {op}"))),
        }
    }
}

impl Command {
    pub fn exec(self) -> anyhow::Result<String> {
        match self {
            Self::Simple(op, arg) => Ok(op.exec(&arg) + "\n"),
            Self::File(op, path) => op.exec(path),
        }
    }
}
