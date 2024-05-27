use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    net, path,
};

fn main() -> anyhow::Result<()> {
    let server_addr = rust_course::get_server_addr(std::env::args().nth(1).as_deref())?;

    let listener = net::TcpListener::bind(server_addr)?;
    eprintln!("Listening on {server_addr}");

    let mut clients =
        HashMap::<net::SocketAddr, std::thread::JoinHandle<anyhow::Result<()>>>::new();

    loop {
        {
            let mut finished_clients = vec![];
            for (client_addr, handle) in clients.iter_mut() {
                if handle.is_finished() {
                    finished_clients.push(*client_addr);
                }
            }

            let num_finished = finished_clients.len();

            for finished_client_addr in finished_clients {
                if let Some(handle) = clients.remove(&finished_client_addr) {
                    handle.join().expect("Couldn't join client thread")?;
                }
            }

            if num_finished > 0 {
                eprintln!("Joined {num_finished} client threads");
            }
        }

        let (mut client_stream, client_addr) = listener.accept()?;

        let handle = std::thread::spawn(move || {
            eprintln!("Handling connection from {client_addr}");

            loop {
                match read_message(&mut client_stream) {
                    // If we failed to execute a valid message, propagate error further
                    Ok(msg) => execute_message(msg, &client_addr)?,
                    Err(err) => {
                        // Don't propagate client errors, just stop reading
                        eprintln!("Failed to read message from {client_addr}: {err}");
                        break;
                    }
                }
            }

            eprintln!("Closing connection to {client_addr}");

            Ok(())
        });

        clients.insert(client_addr, handle);
    }
}

fn execute_message(msg: rust_course::Message, client_addr: &net::SocketAddr) -> anyhow::Result<()> {
    use rust_course::Message;

    match msg {
        Message::File(filename, data) => {
            let filepath = path::Path::new("./files").join(&filename);
            receive_file(&filepath, &data)?;

            eprintln!(
                "Received file {filename} from {client_addr} ({} bytes) to {:?}",
                data.len(),
                filepath
            );
        }
        Message::Image(filename, data) => {
            let filepath = path::Path::new("./images").join(&filename);
            receive_file(&filepath, &data)?;

            eprintln!(
                "Received image {filename} from {client_addr} ({} bytes) to {:?}",
                data.len(),
                filepath
            );
        }
        Message::Text(msg) => {
            println!("Message from {client_addr}: {msg}");
        }
    }

    Ok(())
}

fn receive_file(filepath: &path::Path, data: &[u8]) -> anyhow::Result<()> {
    let mut file = fs::File::create(filepath)?;
    file.write_all(data)?;

    Ok(())
}

fn read_message(client_stream: &mut net::TcpStream) -> anyhow::Result<rust_course::Message> {
    let mut len_bytes = [0u8; std::mem::size_of::<rust_course::Len>()];
    client_stream.read_exact(&mut len_bytes)?;

    let len: usize = rust_course::Len::from_be_bytes(len_bytes).try_into()?;
    let mut message_bytes = vec![0u8; len];

    client_stream.read_exact(&mut message_bytes)?;

    serde_cbor::from_slice(&message_bytes).map_err(anyhow::Error::from)
}
