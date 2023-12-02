use std::{io, sync::mpsc, thread};

use anyhow::Result;
use clap::Parser;
use crossterm::{
    execute,
    style::Color,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use client::{
    args::Args, errors::ReceiveMessageError, handle_receive_message, handle_send_message,
    print_colored_string_to_stdout,
};
use libs::{
    message::{Message, MessageType},
    receiver::MessageReceiver,
    remove_new_line,
    sender::MessageSender,
};

fn main() -> Result<()> {
    // enter alternate screen in command line
    execute!(io::stdout(), EnterAlternateScreen)?;

    let args = Args::parse();

    // connect to specified address and port
    let mut sender = MessageSender::new((args.hostname, args.port))?;
    let mut receiver = MessageReceiver::from_message_sender(&sender)?;

    //let tcp_stream = TcpStream::connect((args.hostname, args.port))?;
    let (tx_sender_to_receiver, rx_sender_to_receiver) = mpsc::channel();
    let (tx_receiver_to_sender, rx_receiver_to_sender) = mpsc::channel();

    let handle;

    // if user set username, send message with username from args
    let username_message = ".username ".to_string() + args.username.as_str();
    match sender.send_message(&Message::from(MessageType::UserNameChange(
        username_message,
    ))) {
        Ok(_) => println!("username set"),
        Err(e) => println!("username fail: {:?}", e),
    }

    {
        // thread for handling messages other clients
        handle = thread::spawn(move || loop {
            if let Ok(_) = rx_sender_to_receiver.try_recv() {
                break;
            }

            if let Err(e) = handle_receive_message(&mut receiver) {
                match e {
                    ReceiveMessageError::Server => {
                        let _ = tx_receiver_to_sender.send(true);

                        // in sender handler is waiting on io::stdin().read_line method so user has
                        // to press enter
                        println!("Press 'enter' to close client.");

                        break;
                    }
                    _ => {}
                }
            }
        });
    }

    // loop for handling user input
    loop {
        if let Ok(_) = rx_receiver_to_sender.try_recv() {
            break;
        }

        let mut input = String::new();

        if let Err(_) = io::stdin().read_line(&mut input) {
            continue;
        }

        remove_new_line(&mut input);

        let should_quit = match handle_send_message(&mut sender, &input) {
            Ok(o) => o,
            Err(e) => {
                print_colored_string_to_stdout(e.to_string().as_str(), Color::Red)?;
                continue;
            }
        };

        if should_quit {
            let _ = tx_sender_to_receiver.send(true);
            break;
        }
    }

    let _ = handle.join();

    // leave alternate screen in command line
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
