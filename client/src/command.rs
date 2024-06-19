use std::path;

#[derive(Debug)]
pub enum Command {
    File(path::PathBuf),
    Image(path::PathBuf),
    Message(String),
    AnnounceNickname(String),
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

        if let Some(nickname) = s.strip_prefix(".nick ") {
            return Self::AnnounceNickname(nickname.to_string());
        }

        Self::Message(s.to_string())
    }
}
