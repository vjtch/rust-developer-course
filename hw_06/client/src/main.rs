use std::error::Error;
use std::io;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use client::{handle_receive_message, handle_send_message, Args};
use libs::remove_new_line;

fn main() -> Result<(), Box<dyn Error>> {
    // enter alternate screen in command line
    execute!(io::stdout(), EnterAlternateScreen)?;

    let args = Args::parse();

    // <hostname>:<port>
    let server_address = args.hostname + ":" + args.port.to_string().as_str();

    // connect to specified address and port
    let tcp_stream = TcpStream::connect(server_address)?;
    let (tx, rx) = mpsc::channel();

    let handle;

    // if user set username, send message with username from args
    let username_message = ".username ".to_string() + args.username.as_str();
    let _ = handle_send_message(tcp_stream.try_clone().unwrap(), &username_message);

    {
        let tcp_stream = tcp_stream.try_clone()?;

        // thread for handling messages other clients
        handle = thread::spawn(move || loop {
            if let Ok(_) = rx.try_recv() {
                break;
            }

            if let Ok(ok_tcp_stream) = tcp_stream.try_clone() {
                let _ = handle_receive_message(ok_tcp_stream);
            }
        });
    }

    // loop for handling user input
    loop {
        let tcp_stream = tcp_stream.try_clone()?;

        let mut input = String::new();

        if let Err(_) = io::stdin().read_line(&mut input) {
            continue;
        }

        remove_new_line(&mut input);

        if let Ok(should_quit) = handle_send_message(tcp_stream, &input) {
            if should_quit {
                let _ = tx.send(true);
                break;
            }
        }
    }

    let _ = handle.join();

    // leave alternate screen in command line
    execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
