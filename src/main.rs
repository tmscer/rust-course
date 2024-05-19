use std::{io, process::ExitCode, str::FromStr, thread};

mod cmd;
mod file_format_ops;
mod simple_ops;

fn main() -> anyhow::Result<ExitCode> {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<cmd::Command>();

    let input_thread = thread::spawn(move || {
        use io::BufRead;

        let input = std::io::stdin().lock().lines();
        read_input(input, cmd_tx)
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

fn read_input<R>(
    lines: io::Lines<R>,
    cmd_tx: std::sync::mpsc::Sender<cmd::Command>,
) -> anyhow::Result<usize>
where
    R: io::BufRead,
{
    let mut num_errors = 0;

    for line in lines {
        let line = line?;

        match cmd::Command::from_str(&line) {
            Ok(cmd) => cmd_tx.send(cmd)?,
            Err(err) => {
                num_errors += 1;
                eprintln!("Input error: {err}");
            }
        }
    }

    Ok(num_errors)
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
