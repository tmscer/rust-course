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
