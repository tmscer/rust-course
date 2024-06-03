# Rust Course

## Client-Server Application

This project contains a client-server application for sending messages and files.
See individual crates

- common (lib),
- client (bin),
- server (bin),

for details.

### Common

Defines protocol for sending messages and files and how to write/read to/from network, CLI arguments
and other utilities used by both `client` and `server`.

### Client

To run the client do the following in its directory:

```console
./client $ cargo run -- <server-address>
```

where `server-address` is in format `ip:port`, default is `localhost:11111`.

Commands are read from stdin and sent to the server. They have the following syntax:

```
.file <file-path>  # send file
.image <file-path> # send image
<anything else>    # send text message
```

### Server

To run the server do the following in its directory:

```console
./server $ cargo run -- <server-address>
```

where `server-address` is in format `ip:port`, default is `localhost:11111`.

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
