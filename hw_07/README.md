# Usage

## Server

- run application (`./hw_06/server`):
    - `cargo run`
- arguments:
    - `hostname` - string
    - `port` - unsigned 16 bit integer
- run application with arguments example (`./hw_06/server`):
    - `cargo run -- --hostname localhost --port 8333`
- commands in application:
    - `.quit` - stops application

## Client

- run application (`./hw_06/client`):
    - `cargo run`
- arguments:
    - `hostname` - string
    - `port` - unsigned 16 bit integer
    - `username` - string
- run application with arguments example (`./hw_06/client`):
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

- `lib.rs` code was splitted into multiple files
- message reading and writing is newly done by `MessageSender` and `MessageReceiver`
- `Message` structure contains `serialize` and `deserialize` methods

## Server

- `lib.rs` code was splitted into multiple files
- sends error messages to clients in new `MessageType`s (`RecoverableError` or `UnrecoverableError`)

## Client

- `lib.rs` code was splitted into multiple files
- added handling of errors from server. If `MessageType` is `RecoverableError` client only prints error message and if there is `UnrecoverableError` client also stops (after pressing the enter key).
