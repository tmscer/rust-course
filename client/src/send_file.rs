use std::path;

use common::proto;

use crate::Error;

pub async fn send_stream_file<S>(conn: &mut S, filepath: &path::Path) -> Result<usize, Error>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::AsyncReadExt;

    let mut reader = tokio::fs::File::open(&filepath)
        .await
        .map(tokio::io::BufReader::new)
        .map_err(Error::soft)?;

    let mut buf = vec![0; 4096];
    let mut bytes_sent = 0;
    let mut bytes_file = 0;

    let start = tokio::time::Instant::now();

    loop {
        let bytes_read = reader.read(&mut buf).await.map_err(Error::soft)?;

        if bytes_read == 0 {
            break;
        }

        let message = proto::request::StreamedFile::Payload(buf[..bytes_read].to_vec());

        bytes_file += bytes_read;
        bytes_sent += proto::Payload::new(message)
            .write_to(conn)
            .await
            .map_err(Error::hard)?;

        tracing::debug!("Sent {bytes_sent} bytes, chunk size was {bytes_read}");
    }

    proto::Payload::new(proto::request::StreamedFile::End)
        .write_to(conn)
        .await
        .map_err(Error::hard)?;

    tracing::debug!("Sent the end of file marker");
    let speed = bytes_file as f64 / start.elapsed().as_secs_f64();

    tracing::debug!(
        "Sent {} of data in total, speed was {}/s",
        human_bytes::human_bytes(bytes_file as f64),
        human_bytes::human_bytes(speed),
    );

    Ok(bytes_sent)
}
