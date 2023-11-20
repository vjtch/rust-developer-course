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

- `SystemMessageType` was removed and now are all possible options of messages in `MessageType`
- new structure `Message` was added. It contains `MessageType`, `UserInfo` and `SystemTime` (which is not used now). `UserInfo` contains user's name and color. Structure `Message` is now send between server and clients instead of `MessageType`

## Server

- in `main.rs` is only `main` function. Other functions, structures and enums were moved to `lib.rs`
- all errors and some useful informations are logged and printed to command line. For this functionality I used [tracing](https://crates.io/crates/tracing) and [tracing_subscriber](https://crates.io/crates/tracing-subscriber) crates
- messages from connected clients are handles in multiple threads using [rayon](https://crates.io/crates/rayon) crate

## Client

- in `main.rs` is only `main` function. Other functions, structures and enums were moved to `lib.rs`
- `FromStr` implementation for `CommandType` is newly done with [regex](https://crates.io/crates/regex) crate
- user's can now choose color of their name and also whole chat is displayed in new screen in command line. Both features were done using [crossterm](https://crates.io/crates/crossterm) crate
- when user receives image, it is printed to command line. Size of printed image is determined by window size. This was done using [viuer](https://crates.io/crates/viuer) crate
