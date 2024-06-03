use std::io;

type Len = u64;

/// Represents a message client sends to server. It can serialize itself
/// into a byte stream and deserialize from a byte stream using `Message::write_to`
/// and `Message::read_from` methods.
///
/// The format of the byte stream is as follows:
///
/// 1. 8 bytes representing the length of the payload in big-endian format.
/// 2. `serde_cbor`-encoded representation of the message itself of length from 1.
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Message {
    File(String, Vec<u8>),
    Image(String, Vec<u8>),
    Text(String),
}

impl Message {
    /// Write into a writer.
    ///
    /// # Errors
    ///
    /// If the serialization of the message fails, an error is returned.
    /// This can happen e.g. if your architecture's `usize` cannot contain `u64`.
    pub fn write_to<W>(&self, output: &mut W) -> anyhow::Result<usize>
    where
        W: io::Write,
    {
        let payload = serde_cbor::to_vec(&self)?;
        let len: Len = payload.len().try_into()?;

        output.write_all(&len.to_be_bytes())?;
        output.write_all(&payload)?;

        let bytes_sent = payload.len() + std::mem::size_of::<Len>();

        Ok(bytes_sent)
    }

    /// Read from a reader.
    ///
    /// # Errors
    ///
    /// If the deserialization of the message fails, an error is returned.
    /// This can happen e.g. if your architecture's `usize` cannot contain `u64`.
    pub fn read_from<R>(input: &mut R) -> anyhow::Result<Self>
    where
        R: io::Read,
    {
        let mut len_bytes = [0u8; std::mem::size_of::<Len>()];
        input.read_exact(&mut len_bytes)?;

        let len: usize = Len::from_be_bytes(len_bytes).try_into()?;
        let mut payload = vec![0u8; len];
        input.read_exact(&mut payload)?;

        serde_cbor::from_slice(&payload).map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_and_read() {
        let input_msg = Message::Text("hello!!!!".to_string());

        let mut wire = vec![];
        input_msg.write_to(&mut wire).unwrap();

        let mut cursor = io::Cursor::new(wire);
        let output_msg = Message::read_from(&mut cursor).unwrap();

        assert_eq!(output_msg, input_msg);
    }
}
