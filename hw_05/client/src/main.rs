use std::error::Error;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::str::FromStr;
use std::sync::mpsc;
use std::thread;

use chrono::Utc;
use clap::Parser;
use image::io::Reader;

use libs::{read_message, remove_new_line, send_message, MessageType, SystemMessageType};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // <hostname>:<port>
    let server_address = args.hostname + ":" + args.port.to_string().as_str();

    let tcp_stream = TcpStream::connect(server_address)?;
    let (tx, rx) = mpsc::channel();

    let handle;

    {
        let tcp_stream = tcp_stream.try_clone()?;

        handle = thread::spawn(move || loop {
            if let Ok(_) = rx.try_recv() {
                break;
            }

            if let Ok(ok_tcp_stream) = tcp_stream.try_clone() {
                let _ = handle_receive_message(ok_tcp_stream);
            }
        });
    }

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

    Ok(())
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 11111)]
    port: u16,

    #[arg(long, default_value = "localhost")]
    hostname: String,

    #[arg(long, default_value = "user")]
    username: String,
}

#[derive(PartialEq)]
enum CommandType {
    File,
    Image,
    Text,
    Quit,
    Username,
}

impl FromStr for CommandType {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        if string.starts_with(".file") {
            return Ok(CommandType::File);
        }

        if string.starts_with(".image") {
            return Ok(CommandType::Image);
        }

        if string.starts_with(".quit") {
            return Ok(CommandType::Quit);
        }

        if string.starts_with(".username") {
            return Ok(CommandType::Username);
        }

        return Ok(CommandType::Text);
    }
}

fn handle_send_message(mut stream: TcpStream, input: &str) -> Result<bool, Box<dyn Error>> {
    let command_type = CommandType::from_str(&input)?;

    let message = match command_type {
        CommandType::Text => MessageType::Text(input.to_string()),
        CommandType::File | CommandType::Image => {
            let parts: Vec<&str> = input.split(' ').collect();

            if parts.len() != 2 {
                return Err(r#"Invalid command format. Try this: ".file <filename>""#.into());
            }

            let file_path = parts[1];
            let mut file = File::open(file_path)?;

            let mut buffer_send = vec![];
            file.read_to_end(&mut buffer_send)?;

            match command_type {
                CommandType::File => {
                    let file_name = match Path::new(file_path).file_name() {
                        Some(s) => match s.to_str() {
                            Some(s) => s.to_string(),
                            None => return Err("Could convert file name to String.".into()),
                        },
                        None => return Err("Could not parse file name.".into()),
                    };

                    MessageType::File(file_name, buffer_send)
                }
                CommandType::Image => MessageType::Image(buffer_send),
                _ => {
                    return Err("This should never happen.".into());
                }
            }
        }
        CommandType::Quit => {
            return Ok(true);
        }
        CommandType::Username => {
            let parts: Vec<&str> = input.split(' ').collect();

            if parts.len() != 2 {
                return Err(
                    r#"Invalid command format. Try this: ".username <new username>""#.into(),
                );
            }

            let new_username = parts[1];

            MessageType::System(SystemMessageType::UsernameChange(new_username.to_string()))
        }
    };

    send_message(&mut stream, &message)?;

    Ok(false)
}

fn handle_receive_message(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let message = read_message(&mut stream)?;

    match message {
        MessageType::Text(s) => {
            println!("{}", s);
        }
        MessageType::File(file_name, data) => {
            println!("Receiving {}", file_name);

            let file_extension = match Path::new(&file_name).extension() {
                Some(s) => match s.to_str() {
                    Some(s) => s,
                    None => {
                        return Err("Could not parse file extension.".into());
                    }
                },
                None => {
                    return Err("Could not parse file extension.".into());
                }
            };

            let mut my_file_name = "./files/".to_string();
            my_file_name =
                my_file_name + Utc::now().timestamp().to_string().as_str() + "." + file_extension;

            let mut file = File::create(my_file_name)?;

            let _ = file.write_all(data.as_slice());
        }
        MessageType::Image(data) => {
            println!("Receiving image...");

            let mut my_file_name = "./images/".to_string();
            my_file_name = my_file_name + Utc::now().timestamp().to_string().as_str() + ".png";

            let img = Reader::new(Cursor::new(data))
                .with_guessed_format()?
                .decode()?;

            let _ = img.save_with_format(my_file_name, image::ImageFormat::Png);
        }
        MessageType::System(SystemMessageType::UserConnect(text))
        | MessageType::System(SystemMessageType::UserDisconnect(text))
        | MessageType::System(SystemMessageType::UsernameChange(text)) => {
            println!("{text}");
        }
    }

    Ok(())
}
