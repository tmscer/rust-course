use std::{collections::HashMap, net};

mod handle_client;
mod run;

pub struct Server {
    listener: tokio::net::TcpListener,
    clients: HashMap<net::SocketAddr, tokio::task::JoinHandle<anyhow::Result<()>>>,
}

impl Server {
    pub fn new(listener: tokio::net::TcpListener) -> Self {
        Self {
            listener,
            clients: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Client {
    stream: tokio::net::TcpStream,
    nickname: Option<String>,
}

impl Client {
    pub fn new(tcp_stream: tokio::net::TcpStream) -> Self {
        Self {
            stream: tcp_stream,
            nickname: None,
        }
    }

    pub fn get_stream(&mut self) -> &mut tokio::net::TcpStream {
        &mut self.stream
    }

    pub fn set_nickname(&mut self, nickname: impl ToString) {
        self.nickname = Some(nickname.to_string());
    }

    pub fn get_nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }
}
