use std::{collections::HashMap, io, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tokio::{net::TcpListener, sync::broadcast, sync::Mutex};
use tracing_subscriber;

use libs::remove_new_line;
use server::{args::Args, handle_new_clients};

#[tokio::main]
async fn main() -> Result<()> {
    // turn on printing of logs to command line
    tracing_subscriber::fmt().pretty().init();

    // parse program arguments
    let args = Args::parse();

    // <hostname>:<port>
    let server_address = args.hostname + ":" + args.port.to_string().as_str();

    //  hash map for storing informations (SocketAddr and TcpStream) about connected clients
    let mut connected_clients = Arc::new(Mutex::new(HashMap::new()));

    // broadcast channel for notifying tasks that they should stop
    let (tx, _) = broadcast::channel(8);

    // create tcp connection on specified address and port
    let tcp_listener = TcpListener::bind(server_address).await?;

    // task handle
    let handle;

    // create task for accepting new connections
    {
        let mut tx = tx.clone();

        handle = tokio::spawn(async move {
            handle_new_clients(tcp_listener, &mut connected_clients, &mut tx).await;
        });
    }

    // loop waiting for ".quit" input that terminates server
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        remove_new_line(&mut input);

        if input.eq(".quit") {
            tx.send(true).unwrap();

            break;
        }
    }

    // wait until client handler ends
    handle.await.unwrap();

    Ok(())
}
