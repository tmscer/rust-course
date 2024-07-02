use std::net;

pub const DEFAULT_WEB_SERVER_ADDRESS: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::LOCALHOST), 8080);

#[derive(clap::Parser, Debug, Clone)]
pub struct Config {
    #[clap(long = "web_address", value_parser(common::cli::parse_socket_addr), default_value_t = DEFAULT_WEB_SERVER_ADDRESS)]
    pub web_address: net::SocketAddr,
    #[clap(long = "actix_num_workers", default_value = "2")]
    pub actix_num_workers: usize,
    #[clap(long, default_value = "false")]
    pub disable_docs: bool,
}
