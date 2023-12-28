use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};

use tokio::{
    net::TcpListener, select, sync::broadcast::Receiver, sync::broadcast::Sender, sync::Mutex,
    task::JoinHandle,
};
use tracing::{error, info};

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
                let message_receiver = receiver_sender_builder.message_receiver();
                let message_sender = receiver_sender_builder.message_sender();

                // adding new client into clients list
                match clients.lock().await {
                    mut clients => {
                        info!("Client connected.");

                        clients.insert(
                            addr,
                            Client {
                                message_sender: message_sender.clone(),
                                user_info: UserInfo {
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
                    let rx = tx.subscribe();

                    handles.push(tokio::spawn(async move {
                        handle_connected_client(addr, message_receiver, message_sender, &mut clients, rx)
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
    mut message_receiver: MessageReceiver,
    mut message_sender: MessageSender,
    clients: &mut Arc<Mutex<HashMap<SocketAddr, Client>>>,
    mut rx: Receiver<bool>,
) {
    let mut user = Client {
        message_sender: message_sender.clone(),
        user_info: UserInfo {
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
                    // TODO: refactor! (code below should be more simple, now behaves like managing
                    //                  more connected clients but actually managing only one)

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

                    // send messages to all connected clients (message author will not receive his message)
                    for (addr_target, user) in clients.lock().await.iter_mut() {
                        // create clone of messages list
                        let messages_to_send = match messages_to_send.lock().await {
                            ok_messages_to_send => ok_messages_to_send.clone(),
                        };

                        // send messages to client
                        for (addr_message_author, message) in messages_to_send {
                            if *addr_target == addr_message_author {
                                continue;
                            }

                            let sender = user.message_sender.clone();

                            if let Err(e) = sender.clone().send_message(&message).await {
                                error!("Could not send message: {}", e);
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
    };

    let message_template = Message {
        message: message_type,
        user_info: user.user_info.clone(),
        datetime: message.datetime,
    };

    messages_to_send.lock().await.push((addr, message_template));

    Ok(())
}
