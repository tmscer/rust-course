use std::{env, io, process::ExitCode, str::FromStr, thread};

mod cmd;
mod file_format_ops;
mod simple_ops;

fn main() -> anyhow::Result<ExitCode> {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<cmd::Command>();

    let input_thread = thread::spawn(move || {
        let stdin = io::stdin().lock();
        let input = if let Some(op) = env::args().nth(1) {
            Input::OpAndReader(op, stdin)
        } else {
            Input::Reader(stdin)
        };

        input.send_commands(cmd_tx)
    });

    // Processing thread shall end when the command channel is closed.
    let processing_thread = thread::spawn(move || process_commands(cmd_rx.iter()));

    let num_input_errs = input_thread.join().expect("Failed to join input thread")?;
    let num_processing_errs = processing_thread
        .join()
        .expect("Failed to join processing thread")?;

    let num_errs = num_input_errs + num_processing_errs;

    if num_errs > 0 {
        eprintln!("Encountered one or more errors");
        return Ok(ExitCode::FAILURE);
    }

    Ok(ExitCode::SUCCESS)
}

enum Input<R: io::BufRead> {
    Reader(R),
    OpAndReader(String, R),
}

impl<R: io::BufRead> Input<R> {
    pub fn send_commands(
        self,
        cmd_tx: std::sync::mpsc::Sender<cmd::Command>,
    ) -> anyhow::Result<usize> {
        let cmd_iter: Box<dyn Iterator<Item = anyhow::Result<cmd::Command>>> = match self {
            Self::Reader(reader) => {
                let iter = reader.lines().map(|line_result| {
                    line_result
                        .map_err(anyhow::Error::from)
                        .and_then(|line| cmd::Command::from_str(&line))
                });

                Box::new(iter)
            }
            Self::OpAndReader(op, reader) => {
                let cmd_result = cmd::Command::from_op_name_and_input(&op, reader);
                let iter = std::iter::once(cmd_result);

                Box::new(iter)
            }
        };

        let mut num_errors = 0;

        for cmd_result in cmd_iter {
            match cmd_result {
                Ok(cmd) => {
                    cmd_tx.send(cmd)?;
                }
                Err(err) => {
                    num_errors += 1;
                    eprintln!("Error: {err}");
                }
            }
        }

        Ok(num_errors)
    }
}

fn process_commands(cmd_rx: impl Iterator<Item = cmd::Command>) -> anyhow::Result<usize> {
    let mut num_errors = 0;

    for cmd in cmd_rx {
        match cmd.exec() {
            Ok(output) => print!("{output}"),
            Err(err) => {
                num_errors += 1;
                eprintln!("Processing error: {err}");
            }
        }
    }

    Ok(num_errors)
}
