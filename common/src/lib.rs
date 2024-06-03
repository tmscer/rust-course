pub mod cli;
pub mod tracing;

pub type Len = u64;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub enum Message {
    File(String, Vec<u8>),
    Image(String, Vec<u8>),
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cbor() {
        let text_msg = Message::Text("hello!!!!".to_string());
        let payload = serde_cbor::ser::to_vec(&text_msg).unwrap();

        eprintln!("{payload:?}");
    }
}
