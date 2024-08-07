# Rust Course

## Client-Server Application

This project contains a client-server application for sending messages and files.
See individual crates

- common (lib),
- client (bin),
- server (bin),

for details.

On the high-level, clients send messages and files to server using a custom protocol on one port (default `11111`)
that can run over mTLS.
Server exposes an unsecured (bring your own!) web interface on another port (default `8080`).

The web server also serves autogenerated OpenAPI documentation at [`http://localhost:8080/_docs/redoc`](http://localhost:8080/_docs/redoc).

## Mutual TLS

This repo contains self-signed certs (that are not used anywhere beside here) to get up and
running quickly. They're in directory `./ssl`.

- `ca.crt` - CA certificate
- `ca.key` - CA private key
- `client1.crt` - Client certificate
- `client1.csr` - Client signing request
- `client1.key` - Client private key
- `server-localhost.crt` - Server certificate
- `server-localhost.csr` - Server signing request
- `server-localhost.key` - Server private key
- `server-localhost.bundle.crt` - Server certificate bundle (`cat server-localhost.crt ca.crt > server-localhost.bundle.crt`)

To run without mTLS, disable default features using flag `--no-default-features` as mTLS is enabled by default via feature
"mtls" in both `client` and `server`. Crate common has a feature named `tls`.

> [!NOTE]
>
> I had problems switching between non-TLS and TLS at runtime using `Box<dyn _>`.
> If you know how to do it without enum dispatch and implementing the required traits,
> please let me know.

## Quick Start

In directory `./server`, run command `make setup` to

1. Start Postgres, Prometheus and Grafana using docker-compose.
2. Install [Diesel CLI](https://crates.io/crates/diesel_cli) via Cargo.
3. Run DB migrations.

For more info regarding DB, see [here](#Database).

### Prometheus Metrics

Web server exposes text-encoded metrics at [`http://localhost:8080/metrics`](http://localhost:8080/metrics).
They use the default registry provided by crate [`prometheus`](https://docs.rs/prometheus/0.13.4/prometheus/).

- `http_requests_total`: Total number of HTTP requests over server's lifetime. Excludes metrics and docs requests.
- `messages_total`: Total number of messages handled by server. File streaming messages are counted as one.
- `messages_received_bytes`: Total number of bytes received when handling messages.
- `messages_sent_bytes`: Total number of bytes sent when handling messages.
- `active_connections`: Number of active connections to the server.

### Crate `common`

Defines protocol for sending messages and files and how to write/read to/from network, CLI arguments
and other utilities used by both `client` and `server`. See [./common/src/proto.rs](./common/src/proto.rs) for details.

### Crate `client`

Use `cargo run -- --help` to see usage:

```console
Command-line arguments for the client

Usage: client [OPTIONS] --nick <NICKNAME> [SERVER_ADDRESS]

Arguments:
  [SERVER_ADDRESS]  Server address to bind to or connect to [default: 127.0.0.1:11111]

Options:
  -n, --nick <NICKNAME>            
      --cert-domain <CERT_DOMAIN>  Domain to require from the server [default: localhost]
      --cert <CERT>                Path to the client's certificate [default: ../ssl/client1.crt]
      --key <KEY>                  Path to the client's private key [default: ../ssl/client1.key]
      --ca-cert <CA_CERT>          Path to the CA certificate [default: ../ssl/ca.crt]
  -h, --help                       Print help
```

Commands are read from stdin and sent to the server. They have the following syntax:

```
.file <file-path>    # send file
.image <file-path>   # send image
.nick <new-nickname> # announce nickname to the server
<anything else>      # send text message
```

### Crate `server`

Use `cargo run -- --help` to see usage:

```console
Command-line arguments for the server

Usage: server [OPTIONS] [SERVER_ADDRESS]

Arguments:
  [SERVER_ADDRESS]  Server address to bind to or connect to [default: 127.0.0.1:11111]

Options:
  -r, --root <ROOT>
          [default: .]
      --cert <CERT>
          Path to the server's certificate [default: ../ssl/server-localhost.bundle.crt]
      --key <KEY>
          Path to the server's private key [default: ../ssl/server-localhost.key]
      --ca-cert <CA_CERT>
          Path to the CA certificate used for authenticating clients [default: ../ssl/ca.crt]
      --web_address <WEB_ADDRESS>
          [default: 0.0.0.0:8080]
      --actix_num_workers <ACTIX_NUM_WORKERS>
          [default: 2]
      --disable-docs

  -h, --help
          Print help
```

Besides the arguments, env var `DATABASE_URL` with Postgres connection URL must be set.

Files are saved in `<root-dir>/files` and images are saved in `<root-dir>/images`.
Directories `<root-dir>/files` and `<root-dir>/images` are created if they don't exist.

Server handles connection on the main thread and spawns a new thread for each client.

### Database

[`diesel`](https://crates.io/crates/diesel) and [`diesel_async`](https://crates.io/crates/diesel-async)
are used for user-data persistence on the server. To work with it, see [their guide](https://diesel.rs/guides/getting-started).

Regarding [`schema.rs`](./server/src/schema.rs):

- Each message variant has its own table to simplify new variants in future. Common fields
  are in one common table each variant table references.
- File data is not saved to DB as FS is the natural choice here. DB remembers the file's SHA256
  and path.
- I had trouble convincing Diesel to reference `message_text(message_id) -> message(id)`
  but it would generate `message_text(message_id) -> message(message_id)` so
  I changed column names accordingly.

## Links

- [Course book](https://robot-dreams-rust.mag.wiki)

## Homework

See [tags](https://github.com/tmscer/rust-course/tags) with prefix "hw-".

## Setup

See [`.vscode/settings.json`](./.vscode/settings.json).

|                   |                                                                                    |
| ----------------- | ---------------------------------------------------------------------------------- |
| OS                | GNU/Linux                                                                          |
| Rust Installation | [rustup from package manager](https://archlinux.org/packages/extra/x86_64/rustup/) |
| Editor            | VSCode                                                                             |
| Rust Extensions   | rust-analyzer[^1], crates[^2], CodeLLDB[^3]                                        |
| Other Extensions  | Vim[^4], EditorConfig[^5], Error Lens[^6], GH Copilot[^7]         |

[^1]: https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer
[^2]: https://marketplace.visualstudio.com/items?itemName=serayuzgur.crates
[^3]: https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb
[^4]: https://marketplace.visualstudio.com/items?itemName=vscodevim.vim
[^5]: https://marketplace.visualstudio.com/items?itemName=EditorConfig.EditorConfig
[^6]: https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens
[^7]: https://marketplace.visualstudio.com/items?itemName=GitHub.copilot
