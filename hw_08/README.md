# Usage

## Server

- run application (`./hw_08/server`):
    - `cargo run`
- arguments:
    - `hostname` - string
    - `port` - unsigned 16 bit integer
- run application with arguments example (`./hw_08/server`):
    - `cargo run -- --hostname localhost --port 8333`
- commands in application:
    - `.quit` - stops application

## Client

- run application (`./hw_08/client`):
    - `cargo run`
- arguments:
    - `hostname` - string
    - `port` - unsigned 16 bit integer
    - `username` - string
- run application with arguments example (`./hw_08/client`):
    - `cargo run -- --hostname localhost --port 8333 --username satoshi`
- commands in application:
    - `.file <filename>` - send file to other users
    - `.image <filename>` - send image to other users
    - `.username <new username>` - set name of user
    - `.quit` - stops application
    - `.color <r> <g> <b>` - set color of user's name
    - `<message>` - other strings will be send as messages

# Significant changes

## Library

- `MessageSender` and `MessageReceiver` are asynchronous and owns only `OwnedWriteHalf` or `OwnedReadHalf` of `tokio::net::TcpStream`
- in `builder.rs` `MessageReceiverSenderBuilder` helps create and manage copying `MessageSender` and `MessageReceiver`

## Server

- `main.rs` and `lib.rs` are asynchronous using `Tokio`
- in `main.rs` instead of `Bus` crate using `tokio::sync::broadcast`
- TODO: other changes

## Client

- `main.rs` and `lib.rs` are asynchronous using `Tokio`
- TODO: other changes
