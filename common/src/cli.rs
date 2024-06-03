use std::{net, str::FromStr};

pub const DEFAULT_SERVER_ADDR: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::LOCALHOST), 11_111);

#[derive(clap::Parser)]
pub struct Args {
    #[arg(index = 1, value_parser(parse_socket_addr), default_value_t = DEFAULT_SERVER_ADDR)]
    pub server_address: net::SocketAddr,
}

fn parse_socket_addr(arg: &str) -> anyhow::Result<net::SocketAddr> {
    let s = if let Some(suffix) = arg.strip_prefix("localhost") {
        format!("{localhost}{suffix}", localhost = net::Ipv4Addr::LOCALHOST)
    } else {
        arg.to_string()
    };

    net::SocketAddr::from_str(&s).map_err(anyhow::Error::from)
}
