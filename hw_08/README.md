# Usage

## Prerequisites

- installed and running PostgreSQL server
- installed Diesel CLI ([guide](https://diesel.rs/guides/getting-started))
- correct connection string in `.env` file in root directory of this project (look at `example.env`)
- applied database migrations (command `diesel migration run`)

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
- run application with arguments example (`./hw_08/client`):
    - `cargo run -- --hostname localhost --port 8333`
- login or register to chat
    - `.login <username> <password>` - login to chat
    - `.register <username> <password> <password> <r> <g> <b>` - register new user and login
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
- new `MessageType`s added
- added `hash_password` and `verify_password` functions

## Server

- `main.rs` and `lib.rs` are asynchronous using `Tokio`
- in `main.rs` instead of `Bus` crate using `tokio::sync::broadcast`
- added task for handling inserting of new messages to database
    - task for each client sends new messages to this task via `mpsc` channel
- added logic for logging and registering users and sending old messages

## Client

- `main.rs` and `lib.rs` are asynchronous using `Tokio`
- removed `username` program argument
    - user has to login via command and it is more simple to enter everything in one command than in program arguments
- added new enum `LogRegCommandType` for managing `.login` and `.register` commands
- in `main.rs` and `lib.rs` added logic to login or register
    - in `main.rs` send login or register message to server and waits for response. If the response is ok, then was login or registration successful and user can send messages. Otherwise user has to try to login or register again

## Database

- created migrations
    - first is init migration
    - second creates `users`, `messages` and `colors` tables
- in `models.rs` added methods for inserting and reading records from database
- added function to establish connection with database

## App logic

- client has to login on start
    - client sends login request message to server
    - server check that password is correct and sends login response message to client
    - if password was correct client is logged in, otherwise has to try login again
- registration works similarly
- only text messages are stored in database (not files, images or any system messages)
- password in database is stored hashed (`pbkdf2` crate)
- when client connects, server sends him last 20 messages

## TODOs

- fix
    - handle all errors, remove all unwraps
    - old messages are send to all clients after one client requests them
    - data are not inserted into database pernamently, make them published
    - setting and changing color of username
    - instead of `tokio::sync::broadcast` for sending information about stopping server use `tokio::sync::watch`
    - server `.quit` command
    - create better approach when inserting clients to `Arc<Mutex<HashMap<SocketAddr, Client>>>` in `server\lib.rs`
    - when reading messages from database should order them by `created_at`
- more redable code
    - split `MessageType` to `MessageTypeClientToServer` and `MessageTypeServerToClient`
    - split long functions in shorter ones
    - refactor functions in `server\lib.rs`, there is a lot of old or redundant code
    - move login and register logic from `client\main.rs` to `client\lib.rs`
- features
    - send better formated messages when user logs in or registers
    - optimize database access (when login blocks for long time)
