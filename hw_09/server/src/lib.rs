//! Provides structs and methods for handling client connections.

use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use tokio::{
    net::TcpListener,
    select,
    sync::{
        broadcast::{Receiver, Sender},
        mpsc, Mutex,
    },
    task::JoinHandle,
};
use tracing::{error, info};

use database::{
    establish_connection,
    models::{Color, Message as MessageDb, MessageNew, User, UserNew},
};
use libs::{
    builder::MessageReceiverSenderBuilder,
    message::{Message, MessageType, UserInfo},
    receiver::MessageReceiver,
    sender::MessageSender,
};

/// Program arugments
pub mod args;

/// Structure containing informations about connected client.
///
/// # Fields
///
/// * `message_sender` - structure for sending messages over tcp stream
/// * `user_info` - structure that stores informations about user
///
/// # Example
///
/// ```
/// use libs::builder::MessageReceiverSenderBuilder;
/// use libs::message::UserInfo;
/// use server::Client;
/// use tokio::net::TcpListener;
///
/// #[tokio::main]
/// async fn main() {
///     let listener = TcpListener::bind(("localhost", 11111)).await.unwrap();
///
///     // waits until somebody connects to this tcp listerner
///     let (stream, addr) = listener.accept().await.unwrap();
///
///     let receiver_sender_builder = MessageReceiverSenderBuilder::from_tcp_stream(stream).unwrap();
///
///     let client = Client {
///         message_sender: receiver_sender_builder.message_sender(),
///         user_info: UserInfo {
///             id: 1,
///             username: "Alice".to_string(),
///             color: (255, 255, 255),
///         },
///     };
/// }
/// ```
#[derive(Clone)]
pub struct Client {
    pub message_sender: MessageSender,
    pub user_info: UserInfo,
}

/// Handles connection of new cliets.
///
/// Waits until new client want to connect. When new client occurs creates necessary structures and adds new client
/// to `clients` hash map. Then creates new task for this clients.
///
/// # Arguments
///
/// * `listener` - TcpListener on with waits for new clients to connect
/// * `clients` - Hash map of all connected clients
/// * `tx` - Sender side of broadcast channel for .quit command
/// * `msg_db_tx` - Sender side of multiple producer single consumer channel for sending new messages to database handler
///
/// # Panics
///
/// Panics when can't create `MessageReceiverSenderBuilder` from tcp stream.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use server::handle_new_clients;
/// use tokio::net::TcpListener;
/// use tokio::sync::broadcast;
/// use tokio::sync::mpsc;
/// use tokio::sync::Mutex;
///
/// #[tokio::main]
/// async fn main() {
///     // hash map for storing informations (SocketAddr and TcpStream) about connected clients
///     let mut connected_clients = Arc::new(Mutex::new(HashMap::new()));
///
///     // create tcp connection on specified address and port
///     let listener = TcpListener::bind(("localhost", 11111)).await.unwrap();
///
///     // broadcast channel for notifying tasks that they should stop
///     let (mut tx, _) = broadcast::channel(8);
///
///     // multiple produces and single consumer channel for sending messages to database handler
///     let (mut msg_db_tx, _) = mpsc::channel(64);
///
///     handle_new_clients(listener, &mut connected_clients, &mut tx, &mut msg_db_tx).await;
/// }
/// ```
pub async fn handle_new_clients(
    listener: TcpListener,
    clients: &mut Arc<Mutex<HashMap<SocketAddr, Client>>>,
    tx: &mut Sender<bool>,
    msg_db_tx: &mut mpsc::Sender<Message>,
) {
    let mut rx = tx.subscribe();
    let mut handles: Vec<JoinHandle<()>> = vec![];

    loop {
        select! {
            // check broadcast channel signaling termination
            Ok(_) = rx.recv() => {
                break;
            }
            Ok((stream, addr)) = listener.accept() => {
                let receiver_sender_builder =
                    MessageReceiverSenderBuilder::from_tcp_stream(stream).unwrap();
                let mut message_receiver = receiver_sender_builder.message_receiver();
                let mut message_sender = receiver_sender_builder.message_sender();

                // adding new client into clients list
                match clients.lock().await {
                    mut clients => {
                        info!("Client connected.");

                        clients.insert(
                            addr,
                            Client {
                                message_sender: message_sender.clone(),
                                user_info: UserInfo {
                                    // TODO: fix
                                    id: 0,
                                    username: "<anonymous user>".to_string(),
                                    color: (255, 255, 255),
                                },
                            },
                        );
                    }
                }

                // create task for handling new client
                {
                    let mut clients = clients.clone();
                    let mut rx = tx.subscribe();
                    let mut msg_db_tx = msg_db_tx.clone();

                    handles.push(tokio::spawn(async move {
                        handle_connected_client(addr, &mut message_receiver, &mut message_sender, &mut clients, &mut rx, &mut msg_db_tx)
                            .await;
                    }));
                }
            }
        }
    }

    for handle in handles.iter_mut() {
        handle.await.unwrap();
    }
}

/// Handles connected client.
///
/// Waits until receives message from connected client or gets signal from termination channel.
///
/// # Arguments
///
/// * `addr` - client's socket address
/// * `message_receiver` - client's `MessageReceiver`
/// * `message_sender` - client's `MessageSender`
/// * `clients` - Hash map of all connected clients
/// * `rx` - Receiver side of broadcast channel for .quit command
/// * `msg_db_tx` - Sender side of multiple producer single consumer channel for sending new messages to database handler
///
/// # Panics
///
/// Panics when can't send message to `msg_db_tx` channel.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use libs::builder::MessageReceiverSenderBuilder;
/// use libs::message::UserInfo;
/// use server::Client;
/// use server::handle_connected_client;
/// use tokio::net::TcpListener;
/// use tokio::sync::broadcast;
/// use tokio::sync::mpsc;
/// use tokio::sync::Mutex;
///
/// #[tokio::main]
/// async fn main() {
///     // hash map for storing informations (SocketAddr and TcpStream) about connected clients
///     let mut connected_clients = Arc::new(Mutex::new(HashMap::new()));
///
///     let listener = TcpListener::bind(("localhost", 11111)).await.unwrap();
///
///     // waits until somebody connects to this tcp listerner
///     let (stream, addr) = listener.accept().await.unwrap();
///
///     let receiver_sender_builder = MessageReceiverSenderBuilder::from_tcp_stream(stream).unwrap();
///     let mut message_receiver = receiver_sender_builder.message_receiver();
///     let mut message_sender = receiver_sender_builder.message_sender();
///
///     let mut client = Client {
///         message_sender: message_sender.clone(),
///         user_info: UserInfo {
///             id: 1,
///             username: "Alice".to_string(),
///             color: (255, 255, 255),
///         },
///     };
///
///     connected_clients.lock().await.insert(addr, client.clone());
///
///     // broadcast channel for notifying tasks that they should stop
///     let (_, mut rx) = broadcast::channel(8);
///
///     // multiple produces and single consumer channel for sending messages to database handler
///     let (mut msg_db_tx, _) = mpsc::channel(64);
///
///     handle_connected_client(addr, &mut message_receiver, &mut message_sender, &mut connected_clients, &mut rx, &mut msg_db_tx).await;
/// }
/// ```
pub async fn handle_connected_client(
    addr: SocketAddr,
    message_receiver: &mut MessageReceiver,
    message_sender: &mut MessageSender,
    clients: &mut Arc<Mutex<HashMap<SocketAddr, Client>>>,
    rx: &mut Receiver<bool>,
    msg_db_tx: &mut mpsc::Sender<Message>,
) {
    let mut user = Client {
        message_sender: message_sender.clone(),
        user_info: UserInfo {
            // TODO: fix
            id: 0,
            username: "<anonymous user>".to_string(),
            color: (255, 255, 255),
        },
    };

    loop {
        select! {
            // check broadcast channel signaling termination
            Ok(_) = rx.recv() => {
                let _ = message_sender.send_message(&Message::from(
                    MessageType::UnrecoverableError(
                        "Server is stopped. Try connect later.".to_string(),
                    ),
                )).await;

                info!("Stoped handling connected client.");

                break;
            }
            message = message_receiver.receive_message() => match message {
                Ok(message) => {
                    let mut message = match match_message_type_and_do_server_side_actions(message, &mut user, addr, clients.clone()).await {
                        Ok(m) => m,
                        Err(e) => {
                            error!("Could not process message: {}", e);
                            continue;
                        }
                    };

                    match message.message {
                        // messages to send only to requester
                        MessageType::LoginResponse(_) | MessageType::RegisterResponse(_) | MessageType::OldMessagesResponse(..) => {
                            message_sender.send_message(&message).await.unwrap();
                        }
                        // send messages to all connected clients
                        _ => {
                            for (addr_target, user) in clients.lock().await.iter_mut() {
                                if *addr_target == addr {
                                    match message.message {
                                        // when iterating over client that is author of this message, send
                                        // this message to database handler, also fixes user id
                                        MessageType::Text(_) => {
                                            message.user_info.id = user.user_info.id;

                                            msg_db_tx.send(message.clone()).await.unwrap();
                                        }

                                        _ => {}
                                    }

                                    continue;
                                }

                                let sender = user.message_sender.clone();

                                // do not send login, register and old messages to everyone
                                match message.message {
                                    MessageType::LoginResponse(_) | MessageType::RegisterResponse(_) | MessageType::OldMessagesResponse(..) => {}
                                    _ => {
                                        if let Err(e) = sender.clone().send_message(&message).await {
                                            error!("Could not send message: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Err(_) => {
                    // println!("{:?}", e);
                }
            }
        }
    }
}

/// Matches message type and do server side actions.
///
/// Returns Message with message content and informations about author of message. On some message
/// types does special actions on server side:
/// - `UserNameChange` - changes username of client (`user` variable, not in database)
/// - `UserColorChange` - changes color of client's username (`user` variable, not in database)
/// - `UserDisconnect` - removes this client from `clients` hash map
/// - `LoginRequest` - logs in user and updates data about user in `clients` hash map
/// - `RegisterRequest` - registers user and updates data about user in `clients` hash map
/// - `OldMessagesRequest` - gets last 20 messages from database and returns them
/// - `LoginResponse`, `RegisterResponse`, `OldMessagesResponse` - returns error
///
/// # Arguments
///
/// * `message` - Message to be processed
/// * `user` - Client that send this message
/// * `addr` - Socket address of this client
/// * `clients` - Hash map of all connected clients
///
/// # Panics
///
/// Panics when can't read message, user or color records from database.
///
/// # Example
///
/// ```
/// use std::collections::HashMap;
/// use std::sync::Arc;
/// use libs::builder::MessageReceiverSenderBuilder;
/// use libs::message::UserInfo;
/// use server::Client;
/// use server::match_message_type_and_do_server_side_actions;
/// use tokio::net::TcpListener;
/// use tokio::sync::mpsc;
/// use tokio::sync::Mutex;
///
/// #[tokio::main]
/// async fn main() {
///     // hash map for storing informations (SocketAddr and TcpStream) about connected clients
///     let connected_clients = Arc::new(Mutex::new(HashMap::new()));
///
///     let listener = TcpListener::bind(("localhost", 11111)).await.unwrap();
///
///     // waits until somebody connects to this tcp listerner
///     let (stream, addr) = listener.accept().await.unwrap();
///
///     let receiver_sender_builder = MessageReceiverSenderBuilder::from_tcp_stream(stream).unwrap();
///
///     let mut client = Client {
///         message_sender: receiver_sender_builder.message_sender(),
///         user_info: UserInfo {
///             id: 1,
///             username: "Alice".to_string(),
///             color: (255, 255, 255),
///         },
///     };
///
///     connected_clients.lock().await.insert(addr, client.clone());
///
///     let mut message = match receiver_sender_builder.message_receiver().receive_message().await {
///         Ok(message) => message,
///         Err(_) => { return; },
///     };
///
///     let message = match_message_type_and_do_server_side_actions(message, &mut client, addr, connected_clients).await.unwrap();
/// }
/// ```
pub async fn match_message_type_and_do_server_side_actions(
    message: Message,
    user: &mut Client,
    addr: SocketAddr,
    clients: Arc<Mutex<HashMap<SocketAddr, Client>>>,
) -> Result<Message, Box<dyn Error>> {
    let message_type = match message.message {
        MessageType::UserNameChange(new_username) => {
            let message_type = MessageType::UserNameChange(user.user_info.username.clone());

            user.user_info.username = new_username;

            message_type
        }
        MessageType::UserColorChange(r, g, b) => {
            let message_type = MessageType::UserColorChange(
                user.user_info.color.0,
                user.user_info.color.1,
                user.user_info.color.2,
            );

            user.user_info.color = (r, g, b);

            message_type
        }
        MessageType::Text(_)
        | MessageType::File(..)
        | MessageType::Image(_)
        | MessageType::UserConnect()
        | MessageType::UnrecoverableError(_)
        | MessageType::RecoverableError(_) => message.message,
        MessageType::UserDisconnect() => {
            clients.lock().await.remove(&addr);
            message.message
        }

        MessageType::LoginRequest(username, password) => {
            let connection = &mut establish_connection();

            match User::login(connection, username.as_str(), password.as_str()) {
                Ok(user) => {
                    let color;

                    match user.color_id {
                        Some(color_id) => {
                            let res = Color::read(connection, color_id).unwrap();
                            color = (res.r as u8, res.g as u8, res.b as u8);
                        }
                        None => {
                            color = (255, 255, 255);
                        }
                    }

                    // updates user id in clients (Arc<Mutex<HashMap<SocketAddr, Client>>>)
                    // otherwise there is 0 and when inserting to database throws error
                    // because user with id 0 does not exist
                    if let Some(client) = clients.lock().await.get_mut(&addr) {
                        client.user_info.id = user.id;
                    }

                    MessageType::LoginResponse(Some(UserInfo {
                        id: user.id,
                        username: username,
                        color: color,
                    }))
                }
                Err(_) => MessageType::LoginResponse(None),
            }
        }

        MessageType::RegisterRequest(username, password, r, g, b) => {
            let connection = &mut establish_connection();

            let user = UserNew::register(connection, username.as_str(), password.as_str(), r, g, b);

            let color;

            match user.color_id {
                Some(color_id) => {
                    let res = Color::read(connection, color_id).unwrap();
                    color = (res.r as u8, res.g as u8, res.b as u8);
                }
                None => {
                    color = (255, 255, 255);
                }
            }

            // updates user id in clients (Arc<Mutex<HashMap<SocketAddr, Client>>>)
            // otherwise there is 0 and when inserting to database throws error
            // because user with id 0 does not exist
            if let Some(client) = clients.lock().await.get_mut(&addr) {
                client.user_info.id = user.id;
            }

            MessageType::RegisterResponse(Some(UserInfo {
                id: user.id,
                username: user.username,
                color: color,
            }))
        }

        MessageType::OldMessagesRequest() => {
            let connection = &mut establish_connection();

            let msgs = MessageDb::read(connection, 20).unwrap();
            let mut content = vec![];

            for msg in msgs {
                let user = User::read_by_id(connection, msg.user_id).unwrap();
                let color = Color::read(connection, user.color_id.unwrap()).unwrap();

                content.push((
                    msg.text,
                    UserInfo {
                        id: msg.user_id,
                        username: user.username,
                        color: (color.r as u8, color.g as u8, color.b as u8),
                    },
                ));
            }

            let message_template = Message {
                message: MessageType::OldMessagesResponse(content),
                user_info: user.user_info.clone(),
                datetime: message.datetime,
            };

            return Ok(message_template);
        }

        MessageType::LoginResponse(..)
        | MessageType::RegisterResponse(..)
        | MessageType::OldMessagesResponse(..) => {
            return Err("only server -> client message type".into());
        }
    };

    let message_template = Message {
        message: message_type,
        user_info: user.user_info.clone(),
        datetime: message.datetime,
    };

    Ok(message_template)
}

/// Handles storage of messages received from channel to database.
///
/// Function establishes connection with database. Then cycles endlessly and tries to receive new
/// messages from channel. When new message appears, it is inserted into the database.
///
/// # Arguments
///
/// * `rx` - Receiver side of multi producer single consumer channel
///
/// # Panics
///
/// If it is not possible to establish connection with database or insertion of new record to
/// database fails, function will panic.
///
/// # Example
///
/// ```
/// use server::handle_saving_messages_to_database;
/// use tokio::sync::mpsc;
///
/// #[tokio::main]
/// async fn main() {
///     let (_, rx) = mpsc::channel(8);
///
///     handle_saving_messages_to_database(rx).await;
/// }
/// ```
pub async fn handle_saving_messages_to_database(mut rx: mpsc::Receiver<Message>) {
    let connection = &mut establish_connection();

    loop {
        match rx.recv().await {
            Some(message) => match message.message {
                MessageType::Text(text) => {
                    let message_new = MessageNew {
                        user_id: message.user_info.id,
                        text: text,
                    };

                    message_new.insert(connection).unwrap();
                }
                _ => {}
            },
            None => {}
        }
    }
}
