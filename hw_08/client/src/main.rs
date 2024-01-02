use std::{
    io::{self},
    str::FromStr,
};

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
    args::Args, commands::LogRegCommandType, errors::ReceiveMessageError, handle_receive_message,
    handle_send_message, print_colored_string_to_stdout,
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

    // login or register
    println!("Login or register?");
    println!("- .login <username> <password>");
    println!("- .register <username> <password> <password> <r> <g> <b>");

    let username;
    //let color;

    // loop until is user logged in or registered
    loop {
        let mut buf = Default::default();

        if io::stdin().read_line(&mut buf).is_ok() {
            match LogRegCommandType::from_str(buf.as_str()) {
                Ok(cmd) => match cmd {
                    LogRegCommandType::Login(username_, mut password) => {
                        remove_new_line(&mut password);

                        message_sender
                            .send_message(&Message::from(MessageType::LoginRequest(
                                username_.clone(),
                                password,
                            )))
                            .await
                            .unwrap();

                        match handle_receive_message(&mut message_receiver).await {
                            Ok(_) => {
                                username = username_;
                                break;
                            }
                            Err(_) => {}
                        }

                        // TODO: fix - with code below colored username works fine
                        // match message_receiver.receive_message().await {
                        //     Ok(message) => match message.message {
                        //         MessageType::LoginResponse(res) => {
                        //             let res = res.unwrap();

                        //             username = res.username;
                        //             color = res.color;

                        //             println!("Login was successful.");

                        //             break;
                        //         }
                        //         _ => {
                        //             println!("Login failed.");
                        //         }
                        //     },
                        //     Err(e) => {
                        //         println!("Login failed. {:?}", e);
                        //     }
                        // }
                    }
                    LogRegCommandType::Register(username_, password, repassword, r, g, b) => {
                        if password == repassword {
                            message_sender
                                .send_message(&Message::from(MessageType::RegisterRequest(
                                    username_.clone(),
                                    password,
                                    r,
                                    g,
                                    b,
                                )))
                                .await
                                .unwrap();

                            match handle_receive_message(&mut message_receiver).await {
                                Ok(_) => {
                                    username = username_;
                                    break;
                                }
                                Err(_) => {}
                            }

                            // TODO: fix - with code below colored username works fine
                            // match message_receiver.receive_message().await {
                            //     Ok(message) => match message.message {
                            //         MessageType::RegisterResponse(res) => {
                            //             let res = res.unwrap();

                            //             username = res.username;
                            //             color = res.color;

                            //             println!("Registration was successful.");

                            //             break;
                            //         }
                            //         _ => {
                            //             println!("Registration failed.");
                            //         }
                            //     },
                            //     Err(e) => {
                            //         println!("Registration failed. {:?}", e);
                            //     }
                            // }
                        } else {
                            println!("Passwords do not match.");
                        }
                    }
                },
                Err(e) => {
                    println!("{:?}", e);
                }
            }
        } else {
            println!("Internal error, try to enter command again.");
        }
    }

    // send messages with username and color from login or register
    message_sender
        .send_message(&Message::from(MessageType::UserNameChange(username)))
        .await
        .unwrap();

    // TODO: fix - with code below colored username works fine
    // message_sender
    //     .send_message(&Message::from(MessageType::UserColorChange(
    //         color.0, color.1, color.2,
    //     )))
    //     .await
    //     .unwrap();

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

    // request old messages from server
    message_sender
        .send_message(&Message::from(MessageType::OldMessagesRequest()))
        .await
        .unwrap();

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
