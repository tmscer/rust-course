use std::{cmp::Ordering, path};

use common::proto;
use tokio::io::AsyncWriteExt;

use crate::msg_exec::StreamInfo;

// 1024 was enough during experiments, this should be enough for (hopefully) all
const MIME_DETECTION_BUFFER_SIZE: usize = 4096;

pub async fn receive_streamed_file<H: sha2::Digest, S: tokio::io::AsyncReadExt + Unpin>(
    filepath: &path::PathBuf,
    expected: u64,
    stream: &mut S,
) -> Result<StreamInfo, StreamFileError> {
    let mut file = tokio::fs::File::create(filepath)
        .await
        .map_err(StreamFileError::fs)?;
    let mut received = 0;

    let mut detection_buffer = [0u8; MIME_DETECTION_BUFFER_SIZE];
    // is guaranteed not to be out of bounds
    let mut bytes_in_detection_buffer = 0;

    let mut hasher = H::new();

    while received <= expected {
        match proto::Payload::read_from(stream)
            .await
            .map(|p| p.into_inner())
        {
            Ok(proto::request::StreamedFile::Payload(data)) => {
                file.write_all(&data).await.map_err(StreamFileError::fs)?;
                received += u64::try_from(data.len()).map_err(StreamFileError::read)?;

                hasher.update(&data);
                bytes_in_detection_buffer += copy_bytes(
                    data.as_ref(),
                    &mut detection_buffer[bytes_in_detection_buffer..],
                );
            }
            Ok(proto::request::StreamedFile::Abort) => {
                if let Err(e) = tokio::fs::remove_file(filepath)
                    .await
                    .map_err(StreamFileError::fs)
                {
                    tracing::error!("Failed to remove file due to client abort: {e}");
                }

                return Err(StreamFileError::Abort { expected, received });
            }
            Ok(proto::request::StreamedFile::End) => {
                break;
            }
            Err(e) => {
                return Err(StreamFileError::Read(e));
            }
        }
    }

    let hash = hasher.finalize().to_vec();
    let detection_buffer = &detection_buffer[..bytes_in_detection_buffer.min(received as usize)];

    let info = StreamInfo {
        length: received,
        hash,
        // From `tree_magic_mini` docs:
        // As the magic database files themselves are licensed under the GPL, you must make sure your project uses a compatible license if you enable this behaviour.
        mime: Some(tree_magic_mini::from_u8(detection_buffer).to_string()),
    };

    decide_streamed_file_result(received, expected).map(|_| info)
}

fn decide_streamed_file_result(received: u64, expected: u64) -> Result<(), StreamFileError> {
    match expected.cmp(&received) {
        Ordering::Equal => Ok(()),
        Ordering::Greater => Err(StreamFileError::ExpectedMore { expected, received }),
        Ordering::Less => Err(StreamFileError::ExpectedLess { expected, received }),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StreamFileError {
    #[error("Expected {expected} bytes but received {received} bytes (too many)")]
    ExpectedMore { expected: u64, received: u64 },
    #[error("Expected {expected} bytes but received {received} bytes (not enough)")]
    ExpectedLess { expected: u64, received: u64 },
    #[error("Client explicitly aborted file transfer without `end` message. Received {received} out of {expected} bytes")]
    Abort { received: u64, expected: u64 },
    #[error("File system error: {0}")]
    Fs(anyhow::Error),
    #[error("Client read error: {0}")]
    Read(anyhow::Error),
}

impl StreamFileError {
    fn fs<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Fs(error.into())
    }

    fn read<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::Read(error.into())
    }
}

impl From<StreamFileError> for proto::response::Error {
    fn from(error: StreamFileError) -> Self {
        match error {
            StreamFileError::Fs(e) => Self::MessageExec(e.to_string()),
            StreamFileError::Abort { .. } => Self::ClientAbort,
            // Explicitly listing the errors in case new variants are added.
            // The programmer will have to decide how to handle them instead of
            // automatically converting them to `Read`.
            StreamFileError::Read(_)
            | StreamFileError::ExpectedLess { .. }
            | StreamFileError::ExpectedMore { .. } => Self::Read(error.to_string()),
        }
    }
}

// Copies from `src` to `dest` from index 0 as much as possible respecting length of both
// to not cause an index out of bounds error. Returns the number of bytes copied.
fn copy_bytes(src: &[u8], dest: &mut [u8]) -> usize {
    let length = usize::min(src.len(), dest.len());

    dest[..length].copy_from_slice(&src[..length]);

    length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decide_streamed_file_result_equal() {
        let received = 10;
        let expected = 10;
        let result = decide_streamed_file_result(received, expected);

        assert!(result.is_ok());
    }

    #[test]
    fn test_decide_streamed_file_result_less() {
        let received = 5;
        let expected = 10;
        let result = decide_streamed_file_result(received, expected);

        assert!(matches!(result, Err(StreamFileError::ExpectedMore { .. })));
    }

    #[test]
    fn test_decide_streamed_file_result_more() {
        let received = 15;
        let expected = 10;
        let result = decide_streamed_file_result(received, expected);

        assert!(matches!(result, Err(StreamFileError::ExpectedLess { .. })));
    }

    #[test]
    fn test_copy_bytes() {
        let src = [1, 2, 3, 4, 5];
        let mut dest = [0; 3];

        let copied = copy_bytes(&src, &mut dest);

        assert_eq!(copied, 3);
        assert_eq!(dest, [1, 2, 3]);
    }
}
