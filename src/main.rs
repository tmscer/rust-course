use std::{env, process::ExitCode};

mod simple_ops;

fn main() -> anyhow::Result<ExitCode> {
    let mut args = env::args().take(2);

    args.next().ok_or(anyhow::Error::msg("Missing 0th arg"))?;
    let transmutation = args
        .next()
        .ok_or(anyhow::Error::msg("Missing transmutation arg"))?;

    let transmute = get_text_transmutation(&transmutation)?;

    for line in std::io::stdin().lines() {
        let line = line?;
        let transmuted_line = transmute(&line);
        println!("{transmuted_line}");
    }

    Ok(ExitCode::SUCCESS)
}

fn get_text_transmutation(name: &str) -> anyhow::Result<impl Fn(&str) -> String> {
    let func = match name {
        "lowercase" => simple_ops::lowercase,
        "uppercase" => simple_ops::uppercase,
        "no-spaces" => simple_ops::no_spaces,
        "slugify" => simple_ops::slugify,
        "reverse" => simple_ops::reverse,
        "no-whitespace" => simple_ops::no_whitespace,
        #[cfg(feature = "random")]
        "spongebob" => simple_ops::spongebob,
        _ => return Err(anyhow::Error::msg("Unknown text transmutation")),
    };

    Ok(func)
}
