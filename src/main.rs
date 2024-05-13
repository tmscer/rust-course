use std::{env, io, process::ExitCode};

mod file_format_ops;
mod simple_ops;

pub trait Operation<R: io::BufRead>: Fn(R) -> anyhow::Result<String> {}

fn main() -> anyhow::Result<ExitCode> {
    let mut args = env::args().take(2);

    args.next().ok_or(anyhow::Error::msg("Missing 0th arg"))?;
    let transmutation = args
        .next()
        .ok_or(anyhow::Error::msg("Missing transmutation arg"))?;

    let transmute = get_text_transmutation::<io::StdinLock>(&transmutation)?;
    let output = transmute(io::stdin().lock())?;

    print!("{output}");

    Ok(ExitCode::SUCCESS)
}

fn read_into_str_op<R, F>(func: F) -> impl Fn(R) -> anyhow::Result<String>
where
    R: io::BufRead,
    F: Fn(&str) -> String,
{
    move |reader: R| -> anyhow::Result<String> {
        let mut result = String::new();

        for line in reader.lines() {
            let line = line?;
            let transmutated_line = func(&line);

            result.push_str(&transmutated_line);
            result.push('\n');
        }

        Ok(result)
    }
}

// Boxing should be ok since the function isn't expected to be called many times.
fn get_text_transmutation<R: io::BufRead + 'static>(
    name: &str,
) -> anyhow::Result<Box<dyn Fn(R) -> anyhow::Result<String>>> {
    match name {
        #[cfg(feature = "csv")]
        "csv" => return Ok(Box::new(file_format_ops::csv::<R>)),
        _ => {}
    }

    let func = match name {
        "lowercase" => simple_ops::lowercase,
        "uppercase" => simple_ops::uppercase,
        "no-spaces" => simple_ops::no_spaces,
        "slugify" => simple_ops::slugify,
        "reverse" => simple_ops::reverse,
        "no-whitespace" => simple_ops::no_whitespace,
        #[cfg(feature = "spongebob")]
        "spongebob" => simple_ops::spongebob,
        _ => return Err(anyhow::Error::msg("Unknown text transmutation")),
    };

    Ok(Box::new(read_into_str_op(func)))
}
