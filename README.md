# Rust Course

## Client-Server Application

This project contains a client-server application for sending messages and files.
See individual crates

- common (lib),
- client (bin),
- server (bin),

for details.

## Quick Start

Spin up Postgres using `docker-compose up -d`.

### Common

Defines protocol for sending messages and files and how to write/read to/from network, CLI arguments
and other utilities used by both `client` and `server`. See [./common/src/proto.rs](./common/src/proto.rs) for details.

### Client

Use `cargo run -- --help` to see usage:

```console
Command-line arguments for both client and server. It contains only one argument - server address. Server uses it to bind to a specific address, while client uses it to connect to the server

Usage: client --nick <NICKNAME> [SERVER_ADDRESS]

Arguments:
  [SERVER_ADDRESS]  Server address to bind to or connect to [default: 127.0.0.1:11111]

Options:
  -n, --nick <NICKNAME>
  -h, --help             Print help
```

Commands are read from stdin and sent to the server. They have the following syntax:

```
.file <file-path>    # send file
.image <file-path>   # send image
.nick <new-nickname> # announce nickname to the server
<anything else>      # send text message
```

### Server

Use `cargo run -- --help` to see usage:

```console
Command-line arguments for both client and server. It contains only one argument - server address. Server uses it to bind to a specific address, while client uses it to connect to the server

Usage: server [OPTIONS] [SERVER_ADDRESS]

Arguments:
  [SERVER_ADDRESS]  Server address to bind to or connect to [default: 127.0.0.1:11111]

Options:
  -r, --root <ROOT>  [default: .]
  -h, --help         Print help
```

Files are saved in `<root-dir>/files` and images are saved in `<root-dir>/images`.
Directories `<root-dir>/files` and `<root-dir>/images` are created if they don't exist.

Server handles connection on the main thread and spawns a new thread for each client.

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
