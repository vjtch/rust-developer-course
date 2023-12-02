use std::collections::HashMap;
use std::io;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::Result;
use bus::Bus;
use clap::Parser;
use tracing_subscriber;

use libs::remove_new_line;
use server::{args::Args, handle_connected_clients, handle_new_clients};

fn main() -> Result<()> {
    // turn on printing of logs to command line
    tracing_subscriber::fmt().pretty().init();

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

    // create new tcp connection on specified address and port
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
