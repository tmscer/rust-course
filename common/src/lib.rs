use std::{net, str::FromStr};

pub mod tracing;

pub const DEFAULT_SERVER_ADDR: net::SocketAddrV4 =
    net::SocketAddrV4::new(net::Ipv4Addr::LOCALHOST, 11_111);
pub type Len = u64;

pub fn get_server_addr(arg: Option<&str>) -> anyhow::Result<net::SocketAddrV4> {
    let Some(s) = arg else {
        return Ok(DEFAULT_SERVER_ADDR);
    };

    let s = if let Some(suffix) = s.strip_prefix("localhost") {
        format!("{localhost}{suffix}", localhost = net::Ipv4Addr::LOCALHOST)
    } else {
        s.to_string()
    };

    net::SocketAddrV4::from_str(&s).map_err(anyhow::Error::from)
}

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
