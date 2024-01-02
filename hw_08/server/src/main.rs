use std::{collections::HashMap, io, sync::Arc};

use anyhow::Result;
use clap::Parser;
use tokio::{
    net::TcpListener,
    sync::{broadcast, mpsc, Mutex},
};
use tracing_subscriber;

use libs::remove_new_line;
use server::{args::Args, handle_new_clients, handle_saving_messages_to_database};

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

    // multiple produces and single consumer channel for sending messages to database handler
    let (msg_db_tx, msg_db_rx) = mpsc::channel(64);

    // create tcp connection on specified address and port
    let tcp_listener = TcpListener::bind(server_address).await?;

    // task handle
    let mut handles = vec![];

    // create task for saving text messages to database
    {
        handles.push(tokio::spawn(async move {
            handle_saving_messages_to_database(msg_db_rx).await;
        }));
    }

    // create task for accepting new connections
    {
        let mut tx = tx.clone();
        let mut msg_db_tx = msg_db_tx.clone();

        handles.push(tokio::spawn(async move {
            handle_new_clients(
                tcp_listener,
                &mut connected_clients,
                &mut tx,
                &mut msg_db_tx,
            )
            .await;
        }));
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

    // wait until all tasks are done
    for handle in handles {
        handle.await.unwrap();
    }

    Ok(())
}
