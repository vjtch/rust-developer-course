use std::io::{self};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    style::Color,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use tokio::{
    io::{self as aio, AsyncBufReadExt, BufReader},
    select,
    sync::mpsc,
};

use client::{
    args::Args, errors::ReceiveMessageError, handle_receive_message, handle_send_message,
    print_colored_string_to_stdout,
};
use libs::{
    builder::MessageReceiverSenderBuilder,
    message::{Message, MessageType},
    remove_new_line,
};

#[tokio::main]
async fn main() -> Result<()> {
    // enter alternate screen in command line
    execute!(io::stdout(), EnterAlternateScreen)?;

    // parse program arguments
    let args = Args::parse();

    // connect to specified address and port
    let receiver_sender_builder =
        MessageReceiverSenderBuilder::from_socket_addr((args.hostname, args.port))
            .await
            .unwrap();
    let mut message_receiver = receiver_sender_builder.message_receiver();
    let mut message_sender = receiver_sender_builder.message_sender();

    let (tx_sender_to_receiver, mut rx_sender_to_receiver) = mpsc::channel(1);
    let (tx_receiver_to_sender, mut rx_receiver_to_sender) = mpsc::channel(1);

    // send message with username from args
    match message_sender
        .send_message(&Message::from(MessageType::UserNameChange(args.username)))
        .await
    {
        Ok(_) => println!("username set"),
        Err(e) => println!("username fail: {:?}", e),
    }

    // task for handling messages other clients
    let handle = tokio::spawn(async move {
        loop {
            select! {
                Some(_) = rx_sender_to_receiver.recv() => {
                    break;
                }
                res = handle_receive_message(&mut message_receiver) => match res {
                    Ok(_) => {}
                    Err(e) => {
                        match e {
                            ReceiveMessageError::Server => {
                                tx_receiver_to_sender.send(true).await.unwrap();

                                // in sender handler is waiting on io::stdin().read_line method so user has
                                // to press enter
                                println!("Press 'enter' to close client.");

                                break;
                            }
                            _ => {}
                        }
                    }
                }
            };
        }
    });

    // loop for handling user input
    loop {
        let mut input = String::new();
        let mut reader = BufReader::new(aio::stdin());

        select! {
            Some(_) = rx_receiver_to_sender.recv() => {
                break;
            }
            Ok(_) = reader.read_line(&mut input) => {
                remove_new_line(&mut input);

                let should_quit = match handle_send_message(&mut message_sender, &input).await {
                    Ok(o) => o,
                    Err(e) => {
                        print_colored_string_to_stdout(e.to_string().as_str(), Color::Red)?;
                        continue;
                    }
                };

                if should_quit {
                    tx_sender_to_receiver.send(true).await.unwrap();
                    break;
                }
            }
        }
    }

    // wait until is task completed
    handle.await.unwrap();

    // leave alternate screen in command line
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
