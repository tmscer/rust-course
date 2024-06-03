use std::{collections::HashMap, fs, io::Write, net, path};

use clap::Parser;

fn main() -> anyhow::Result<()> {
    common::tracing::init()?;

    let args = common::cli::Args::parse();
    let listener = net::TcpListener::bind(args.server_address)?;
    tracing::info!("Listening on {}", args.server_address);

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
                tracing::debug!("Joined {num_finished} client threads");
            }
        }

        let (mut client_stream, client_addr) = listener.accept()?;

        let handle = std::thread::spawn(move || {
            tracing::info!("Handling connection from {client_addr}");

            loop {
                match common::proto::Message::read_from(&mut client_stream) {
                    // If we failed to execute a valid message, propagate error further
                    Ok(msg) => execute_message(msg, &client_addr)?,
                    Err(err) => {
                        // Don't propagate client errors, just stop reading
                        tracing::debug!("Failed to read message from {client_addr}: {err}");
                        break;
                    }
                }
            }

            tracing::info!("Closing connection to {client_addr}");

            Ok(())
        });

        clients.insert(client_addr, handle);
    }
}

#[tracing::instrument(skip(msg))]
fn execute_message(
    msg: common::proto::Message,
    client_addr: &net::SocketAddr,
) -> anyhow::Result<()> {
    use common::proto::Message;

    tracing::debug!("Handling message");

    match msg {
        Message::File(filename, data) => {
            let root = path::Path::new("./files");
            fs::create_dir(root)?;
            let filepath = root.join(&filename);
            receive_file(&filepath, &data)?;

            tracing::info!(
                "Received file {filename} from {client_addr} ({} bytes) to {:?}",
                data.len(),
                filepath
            );
        }
        Message::Image(filename, data) => {
            let root = path::Path::new("./images");
            fs::create_dir(root)?;
            let filepath = root.join(&filename);
            receive_file(&filepath, &data)?;

            tracing::info!(
                "Received image {filename} from {client_addr} ({} bytes) to {:?}",
                data.len(),
                filepath
            );
        }
        Message::Text(msg) => {
            tracing::info!("Message from {client_addr}: {msg}");
        }
    }

    Ok(())
}

fn receive_file(filepath: &path::Path, data: &[u8]) -> anyhow::Result<()> {
    let mut file = fs::File::create(filepath)?;
    file.write_all(data)?;

    Ok(())
}
