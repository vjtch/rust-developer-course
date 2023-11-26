use std::collections::HashMap;
use std::error::Error;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use bus::BusReader;
use clap::Parser;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use tracing::{error, info};

use libs::{read_message, send_message, Message, MessageType, UserInfo};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 11111)]
    pub port: u16,

    #[arg(long, default_value = "localhost")]
    pub hostname: String,
}

pub struct User {
    pub tcp_stream: TcpStream,
    pub user_info: UserInfo,
}

pub fn handle_new_clients(
    listener: TcpListener,
    clients: &mut Arc<Mutex<HashMap<SocketAddr, User>>>,
    mut rx: BusReader<bool>,
) {
    for stream in listener.incoming() {
        // check broadcast channel signaling termination
        if let Ok(_) = rx.try_recv() {
            break;
        }

        // adding new client connection into clients hash map. If there is error then the client is
        // not connected
        let stream = match stream {
            Ok(ok_stream) => ok_stream,
            Err(e) => {
                error!("Could not accept new tcp stream: {}", e);
                continue;
            }
        };

        // getting socket address of new tcp stream
        let addr = match stream.peer_addr() {
            Ok(ok_socket_addr) => ok_socket_addr,
            Err(e) => {
                error!("Could not get socket address: {}", e);
                continue;
            }
        };

        // adding new client into clients list
        match clients.lock() {
            Ok(mut clients) => {
                info!("Client connected.");

                clients.insert(
                    addr,
                    User {
                        tcp_stream: stream,
                        user_info: UserInfo {
                            username: "<anonymous user>".to_string(),
                            color: (255, 255, 255),
                        },
                    },
                );
            }
            Err(e) => {
                error!("Could not get access to clients list: {}", e);
                continue;
            }
        }
    }
}

pub fn handle_connected_clients(
    clients: &mut Arc<Mutex<HashMap<SocketAddr, User>>>,
    mut rx: BusReader<bool>,
) {
    loop {
        // check broadcast channel signaling termination
        if let Ok(_) = rx.try_recv() {
            info!("Stoped handling connected clients.");
            break;
        }

        // vectors to temporary save messages to send and witch clients remove from clients list
        let messages_to_send = Arc::new(Mutex::new(vec![]));
        let clients_to_remove = Arc::new(Mutex::new(vec![]));

        // in next block are processed messages from all clients. If there is any error it is only
        // logged. There is this approach because it is not possible to stop handling messages from
        // all clients because one error occured. There should be attemp to notify client that his
        // message could not be delivered or some different reason of failure. Then he would know
        // about this issue and e.g. could try to send the message once more or recconect

        match clients.lock() {
            Ok(mut clients) => {
                // read messages from clients and add them to actions vectors
                clients.par_iter_mut().for_each(|(addr, user)| {
                    let addr = addr.clone();

                    match read_message(&mut user.tcp_stream) {
                        Ok(message) => {
                            if let Err(e) = match_message_type_and_do_actions(
                                message,
                                clients_to_remove.clone(),
                                addr,
                                user,
                                messages_to_send.clone(),
                            ) {
                                error!("Could not process message: {}", e);
                            }
                        }
                        Err(_) => {
                            //error!("Could not read message from tcp stream: {}", e);
                        }
                    }
                });

                // remove disconnected clients
                match clients_to_remove.lock() {
                    Ok(clients_to_remove) => {
                        for socker_addr in clients_to_remove.as_slice() {
                            clients.remove(&socker_addr);
                        }
                    }
                    Err(e) => {
                        error!("Could not get access to clients to remove list: {}", e);
                    }
                }

                // send messages to all connected clients (message author will not receive his message)
                clients.par_iter_mut().for_each(|(addr_target, user)| {
                    // create clone of messages list
                    let messages_to_send = match messages_to_send.lock() {
                        Ok(ok_messages_to_send) => ok_messages_to_send.clone(),
                        Err(e) => {
                            error!("Could not get access to messages to send list: {}", e);
                            return;
                        }
                    };

                    // send messages to client
                    for (addr_message_author, message) in messages_to_send {
                        if *addr_target == addr_message_author {
                            continue;
                        }

                        if let Err(e) = send_message(&mut user.tcp_stream, &message) {
                            error!("Could not send message: {}", e);
                        }
                    }
                });
            }
            Err(e) => {
                error!("Could not get access to clients list: {}", e);
            }
        }
    }
}

fn match_message_type_and_do_actions(
    message: Message,
    clients_to_remove: Arc<Mutex<Vec<SocketAddr>>>,
    addr: SocketAddr,
    user: &mut User,
    messages_to_send: Arc<Mutex<Vec<(SocketAddr, Message)>>>,
) -> Result<(), Box<dyn Error>> {
    let message_type = match message.message {
        MessageType::UserDisconnect() => {
            match clients_to_remove.lock() {
                Ok(mut clients_to_remove) => {
                    clients_to_remove.push(addr);
                }
                Err(e) => {
                    error!("Could not get access to clients to remove list: {}", e);
                }
            }

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

    match messages_to_send.lock() {
        Ok(mut messages_to_send) => {
            messages_to_send.push((addr, message_template));
        }
        Err(e) => {
            error!("Could not get access to messages to send list: {}", e);
        }
    }

    Ok(())
}
