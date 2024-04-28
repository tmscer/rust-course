use std::{env, process::ExitCode};

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
        "lowercase" => |s: &str| s.to_lowercase(),
        "uppercase" => |s: &str| s.to_uppercase(),
        "no-spaces" => |s: &str| s.replace(' ', ""),
        "slugify" => |s: &str| slug::slugify(s),
        "reverse" => |s: &str| s.chars().rev().collect(),
        "no-whitespace" => |s: &str| s.chars().filter(|c| !c.is_whitespace()).collect(),
        _ => return Err(anyhow::Error::msg("Unknown text transmutation")),
    };

    Ok(func)
}
