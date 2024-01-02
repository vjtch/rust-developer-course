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

pub mod args;

pub struct Client {
    pub message_sender: MessageSender,
    pub user_info: UserInfo,
}

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
                    // TODO: refactor (code below should be more simple, now behaves like managing
                    //                 more connected clients but actually managing only one)

                    // vectors to temporary save messages to send and witch clients remove from clients list
                    let messages_to_send = Arc::new(Mutex::new(vec![]));
                    let clients_to_remove = Arc::new(Mutex::new(vec![]));

                    // fill clients to remove
                    if let Err(e) = match_message_type_and_do_actions(
                        message,
                        clients_to_remove.clone(),
                        addr,
                        &mut user,
                        messages_to_send.clone(),
                    )
                    .await
                    {
                        error!("Could not process message: {}", e);
                    }

                    // remove disconnected clients
                    match clients_to_remove.lock().await {
                        clients_to_remove => {
                            for socker_addr in clients_to_remove.as_slice() {
                                clients.lock().await.remove(&socker_addr);
                            }
                        }
                    }

                    // send login, register and old messages to handled client
                    {
                        let messages_to_send = match messages_to_send.lock().await {
                            ok_messages_to_send => ok_messages_to_send.clone(),
                        };

                        for (_, message) in messages_to_send {
                            match message.message.clone() {
                                MessageType::LoginResponse(user) | MessageType::RegisterResponse(user) => {
                                    // updates user id in clients (Arc<Mutex<HashMap<SocketAddr, Client>>>)
                                    // otherwise there is 0 and when inserting to database throws error
                                    // because user with id 0 does not exist
                                    if let Some(client) = clients.lock().await.get_mut(&addr) {
                                        client.user_info.id = user.unwrap().id;
                                    }

                                    message_sender.send_message(&message).await.unwrap();
                                }
                                MessageType::OldMessagesResponse(..) => {
                                    message_sender.send_message(&message).await.unwrap();
                                }
                                // other messages should be send to all clients, this is done below
                                _ => {}
                            }
                        }
                    }

                    // send messages to all connected clients (message author will not receive his message)
                    for (addr_target, user) in clients.lock().await.iter_mut() {
                        // create clone of messages list
                        let messages_to_send = match messages_to_send.lock().await {
                            ok_messages_to_send => ok_messages_to_send.clone(),
                        };

                        // send messages to client
                        for (addr_message_author, mut message) in messages_to_send {
                            if *addr_target == addr_message_author {
                                // when iterating over client that is author of this message, send
                                // this message to database handler, also fixes user id
                                match message.message {
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
                },
                Err(_) => {
                    // println!("{:?}", e);
                }
            }
        }
    }
}

async fn match_message_type_and_do_actions(
    message: Message,
    clients_to_remove: Arc<Mutex<Vec<SocketAddr>>>,
    addr: SocketAddr,
    user: &mut Client,
    messages_to_send: Arc<Mutex<Vec<(SocketAddr, Message)>>>,
) -> Result<(), Box<dyn Error>> {
    let message_type = match message.message {
        MessageType::UnrecoverableError(_) => message.message,
        MessageType::RecoverableError(_) => message.message,
        MessageType::UserDisconnect() => {
            clients_to_remove.lock().await.push(addr);

            MessageType::UserDisconnect()
        }
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
        | MessageType::UserConnect() => message.message,

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

            MessageType::RegisterResponse(Some(UserInfo {
                id: user.id,
                username: user.username,
                color: color,
            }))
        }

        MessageType::LoginResponse(..)
        | MessageType::RegisterResponse(..)
        | MessageType::OldMessagesResponse(..) => {
            // only server -> client
            // should be Err
            return Ok(());
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

            messages_to_send.lock().await.push((addr, message_template));

            return Ok(());
        }
    };

    let message_template = Message {
        message: message_type,
        user_info: user.user_info.clone(),
        datetime: message.datetime,
    };

    messages_to_send.lock().await.push((addr, message_template));

    Ok(())
}

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
