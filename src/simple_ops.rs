use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleOp {
    Lowercase,
    Uppercase,
    NoSpaces,
    Slugify,
    Reverse,
    NoWhitespace,
    #[cfg(feature = "spongebob")]
    Spongebob,
}

impl FromStr for SimpleOp {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "lowercase" => Ok(Self::Lowercase),
            "uppercase" => Ok(Self::Uppercase),
            "no_spaces" => Ok(Self::NoSpaces),
            "slugify" => Ok(Self::Slugify),
            "reverse" => Ok(Self::Reverse),
            "no_whitespace" => Ok(Self::NoWhitespace),
            #[cfg(feature = "spongebob")]
            "spongebob" => Ok(Self::Spongebob),
            _ => Err(anyhow::Error::msg(format!("Unknown operation: {s}"))),
        }
    }
}

impl SimpleOp {
    pub fn exec(self, s: &str) -> String {
        match self {
            Self::Lowercase => lowercase(s),
            Self::Uppercase => uppercase(s),
            Self::NoSpaces => no_spaces(s),
            Self::Slugify => slugify(s),
            Self::Reverse => reverse(s),
            Self::NoWhitespace => no_whitespace(s),
            #[cfg(feature = "spongebob")]
            Self::Spongebob => spongebob(s),
        }
    }
}

pub fn lowercase(s: &str) -> String {
    s.to_lowercase()
}

pub fn uppercase(s: &str) -> String {
    s.to_uppercase()
}

pub fn no_spaces(s: &str) -> String {
    s.replace(' ', "")
}

pub fn slugify(s: &str) -> String {
    slug::slugify(s)
}

pub fn reverse(s: &str) -> String {
    s.chars().rev().collect()
}

pub fn no_whitespace(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

#[cfg(feature = "spongebob")]
pub fn spongebob(s: &str) -> String {
    s.chars()
        .map(|c| {
            if rand::random() {
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}
