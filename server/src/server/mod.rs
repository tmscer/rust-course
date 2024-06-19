use std::{collections::HashMap, net};

mod handle_client;
mod run;

mod listener;
pub use listener::Listener;

pub struct Server<L> {
    listener: L,
    clients: HashMap<net::SocketAddr, tokio::task::JoinHandle<anyhow::Result<()>>>,
}

impl<L> Server<L> {
    pub fn new(listener: L) -> Self {
        Self {
            listener,
            clients: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Client<S> {
    address: net::SocketAddr,
    stream: S,
    nickname: Option<String>,
}

impl<S> Client<S> {
    pub fn new(address: net::SocketAddr, stream: S) -> Self {
        Self {
            address,
            stream,
            nickname: None,
        }
    }

    pub fn get_stream(&mut self) -> &mut S {
        &mut self.stream
    }

    pub fn set_nickname(&mut self, nickname: impl ToString) {
        self.nickname = Some(nickname.to_string());
    }

    pub fn get_nickname(&self) -> Option<&str> {
        self.nickname.as_deref()
    }

    pub fn get_address(&self) -> net::SocketAddr {
        self.address
    }
}
