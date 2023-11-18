use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bus::{Bus, BusReader};
use clap::Parser;

use libs::SystemMessageType;
use libs::{read_message, remove_new_line, send_message, MessageType};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // <hostname>:<port>
    let server_address = args.hostname + ":" + args.port.to_string().as_str();

    //  hash map for storing informations (SocketAddr and TcpStream) about connected clients
    let mut connected_clients = Arc::new(Mutex::new(HashMap::new()));

    // broadcast channel for notifying threads that they should stop
    let mut bus = Bus::new(2);
    let rx_thread_1 = bus.add_rx();
    let rx_thread_2 = bus.add_rx();

    let mut handles = vec![];

    let tcp_listener = TcpListener::bind(server_address)?;
    let local_addr = tcp_listener.local_addr()?;

    {
        let mut connected_clients = connected_clients.clone();

        // thread for handling connected clients
        handles.push(thread::spawn(move || {
            handle_connected_clients(&mut connected_clients, rx_thread_1);
        }));
    }

    // thread for accepting new connections
    handles.push(thread::spawn(move || {
        handle_new_clients(tcp_listener, &mut connected_clients, rx_thread_2);
    }));

    // loop waiting for ".quit" input that terminates server
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        remove_new_line(&mut input);

        if input.eq(".quit") {
            bus.broadcast(true);

            // thread handling new connections is waiting on TcpListener incoming method so it is
            // necessary to "wake up" this thread by connecting to the stream. Then thread tries
            // to read from broadcast channel. To ensure that message will already be in the
            // channel this thread waits 100 ms after broadcast message is send and then "wakes up"
            // target thread
            thread::sleep(Duration::from_millis(100));
            let _ = TcpStream::connect(local_addr);

            break;
        }
    }

    // wait for all threads
    for handle in handles {
        let _ = handle.join();
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 11111)]
    port: u16,

    #[arg(long, default_value = "localhost")]
    hostname: String,
}

struct User {
    tcp_stream: TcpStream,
    username: String,
}

fn handle_new_clients(
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
        let stream = if let Ok(ok_stream) = stream {
            ok_stream
        } else {
            continue;
        };

        let addr = if let Ok(ok_socket_addr) = stream.peer_addr() {
            ok_socket_addr
        } else {
            continue;
        };

        match clients.lock() {
            Ok(mut clients) => {
                clients.insert(
                    addr,
                    User {
                        tcp_stream: stream,
                        username: "<anonymous user>".to_string(),
                    },
                );
                eprintln!("new client connected");
            }
            Err(_) => continue,
        }
    }
}

fn handle_connected_clients(
    clients: &mut Arc<Mutex<HashMap<SocketAddr, User>>>,
    mut rx: BusReader<bool>,
) {
    loop {
        // check broadcast channel signaling termination
        if let Ok(_) = rx.try_recv() {
            break;
        }

        let mut messages_to_send = vec![];
        let mut clients_to_remove = vec![];

        // in next block are processed messages from all clients. If there is any error it is
        // overlooked. There is this approach because it is not possible to stop handling messages
        // from all clients because one error occured. But this errors should be logged somewhere
        // and there should be attemp to notify client that his message could not be delivered or
        // some different reason of failure. Then he would know about this issue and e.g. could try
        // to send the message once more or recconect

        if let Ok(mut clients) = clients.lock() {
            // read messages from clients and add them to actions vectors
            for (addr, user) in clients.iter_mut() {
                let addr = addr.clone();

                if let Ok(message) = read_message(&mut user.tcp_stream) {
                    match message {
                        MessageType::System(SystemMessageType::UserDisconnect(_)) => {
                            clients_to_remove.push(addr);

                            messages_to_send.push((
                                addr,
                                MessageType::System(SystemMessageType::UserDisconnect(
                                    user.username.clone() + "has diconnected.",
                                )),
                            ));

                            todo!("fix");
                        }
                        MessageType::System(SystemMessageType::UserConnect(_)) => {
                            messages_to_send.push((
                                addr,
                                MessageType::System(SystemMessageType::UserConnect(
                                    user.username.clone() + "has connected.",
                                )),
                            ));

                            todo!("fix");
                        }
                        MessageType::System(SystemMessageType::UsernameChange(new_username)) => {
                            messages_to_send.push((
                                addr,
                                MessageType::System(SystemMessageType::UserConnect(
                                    user.username.clone()
                                        + " has changed his name to "
                                        + new_username.clone().as_str()
                                        + ".",
                                )),
                            ));

                            user.username = new_username.clone();
                        }
                        MessageType::Text(text) => {
                            messages_to_send.push((
                                addr,
                                MessageType::Text(user.username.clone() + "> " + text.as_str()),
                            ));
                        }
                        MessageType::File(..) | MessageType::Image(_) => {
                            messages_to_send.push((addr, message.clone()));
                        }
                    }
                }
            }

            // remove disconnected clients
            for socker_addr in clients_to_remove {
                clients.remove(&socker_addr);
            }

            // send messages to all connected clients (message author will not receive his message)
            for (addr_target, user) in clients.iter_mut() {
                for (addr_message_author, message) in messages_to_send.as_slice() {
                    if addr_target == addr_message_author {
                        continue;
                    }

                    let _ = send_message(&mut user.tcp_stream, &message);
                }
            }
        }
    }
}
