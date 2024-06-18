use std::io;

pub mod request;
pub mod response;

type Len = u64;

/// Use for sending types across network. The format of the payload is as follows:
///
/// 1. 8 bytes representing the length of the payload in big-endian format.
/// 2. `serde_cbor`-encoded representation of the message itself of length from 1.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Payload<T>(T);

impl<T> Payload<T> {
    /// Create a new payload.
    pub fn new(payload: T) -> Self {
        Self(payload)
    }

    /// Get the inner value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Payload<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> AsMut<T> for Payload<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: serde::Serialize> Payload<T> {
    /// Write into a writer.
    ///
    /// # Errors
    ///
    /// If the serialization fails, an error is returned.
    /// This can happen e.g. if your architecture's `usize` cannot contain `u64`.
    pub async fn write_to<W>(&self, output: &mut W) -> anyhow::Result<usize>
    where
        W: tokio::io::AsyncWriteExt + Unpin,
    {
        let payload = serde_cbor::to_vec(&self.0)?;
        let len: Len = payload.len().try_into()?;

        output.write_all(&len.to_be_bytes()).await?;
        output.write_all(&payload).await?;

        let bytes_sent = payload.len() + std::mem::size_of::<Len>();

        Ok(bytes_sent)
    }
}

impl<T: serde::de::DeserializeOwned> Payload<T> {
    /// Read from a reader.
    ///
    /// # Errors
    ///
    /// If the deserialization fails, an error is returned.
    /// This can happen e.g. if your architecture's `usize` cannot contain `u64`.
    pub async fn read_from<R>(input: &mut R) -> anyhow::Result<Self>
    where
        R: tokio::io::AsyncReadExt + Unpin,
    {
        let mut len_bytes = [0u8; std::mem::size_of::<Len>()];
        input.read_exact(&mut len_bytes).await?;

        let len: usize = Len::from_be_bytes(len_bytes).try_into()?;
        let mut payload = vec![0u8; len];
        input.read_exact(&mut payload).await?;

        let cursor = io::Cursor::new(payload);

        serde_cbor::from_reader(cursor)
            .map(Self)
            .map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod utils {
    use super::*;

    pub async fn assert_roundtrip_succeeds<T>(input_msg: T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let mut wire = vec![];
        Payload(&input_msg).write_to(&mut wire).await.unwrap();

        let mut cursor = io::Cursor::new(wire);
        let output_msg = Payload::<T>::read_from(&mut cursor).await.unwrap();

        assert_eq!(output_msg.into_inner(), input_msg);
    }
}

#[cfg(test)]
mod request_tests {
    use futures::Future;

    use super::request::*;
    use super::utils::assert_roundtrip_succeeds;

    #[tokio::test]
    async fn test_basic_roundtrip() {
        let input_msg = Message::Text("hello!!!!".to_string());

        assert_roundtrip_succeeds(input_msg).await;
    }

    fn async_prop_test(f: impl Future<Output = ()> + Send + 'static) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();

        rt.block_on(f)
    }

    #[test]
    #[should_panic]
    fn test_async_prop_test_panic() {
        async_prop_test(async {
            panic!("this is a test");
        });
    }

    // Tried crate `proptest_async` but it had compilation issues
    proptest::proptest! {
        #[test]
        fn test_message_roundtrip(text in ".*") {
            async_prop_test(async {
                assert_roundtrip_succeeds(Message::Text(text)).await;
            });
        }

        #[test]
        fn test_file_roundtrip(filename in ".+", payload in proptest::arbitrary::any::<Vec<u8>>()) {
            async_prop_test(async {
                assert_roundtrip_succeeds(Message::File(filename, payload)).await;
            });
        }

        #[test]
        fn test_image_roundtrip(filename in ".+", payload in proptest::arbitrary::any::<Vec<u8>>()) {
            async_prop_test(async {
                assert_roundtrip_succeeds(Message::Image(filename, payload)).await;
            });
        }
    }
}

#[cfg(test)]
mod response_tests {
    use super::response::*;
    use super::utils::assert_roundtrip_succeeds;

    #[tokio::test]
    async fn test_ok() {
        assert_roundtrip_succeeds(Message::Ok).await;
    }

    #[tokio::test]
    async fn test_error() {
        assert_roundtrip_succeeds(Message::Err(Error::unspecified("oops"))).await;
    }
}
