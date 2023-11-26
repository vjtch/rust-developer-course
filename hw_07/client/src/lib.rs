use std::error::Error;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use std::net::TcpStream;
use std::path::Path;
use std::str::FromStr;
use std::time::SystemTime;

use chrono::Utc;
use clap::Parser;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use image::io::Reader;
use regex::Regex;
use viuer::Config;

use libs::{read_message, send_message, Message, MessageType, UserInfo};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long, default_value_t = 11111)]
    pub port: u16,

    #[arg(long, default_value = "localhost")]
    pub hostname: String,

    #[arg(long, default_value = "<anonymous user>")]
    pub username: String,
}

#[derive(PartialEq)]
enum CommandType {
    File(String),
    Image(String),
    Text(String),
    Quit,
    Username(String),
    Color((u8, u8, u8)),
}

impl FromStr for CommandType {
    type Err = &'static str;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        // parse input from user by using regex. There are few options:
        // - .file <filename>
        // - .image <filename>
        // - .username <new username>
        // - .quit
        // - .color <r> <g> <b>
        // - <other text is send as message>
        let re = Regex::new(r"((?<cmd>.file|.image|.username) (?<name>.+)|(?<quit>.quit)|(?<color>.color (?<r>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)) (?<g>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)) (?<b>(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)))|(?<text>.+))").unwrap();
        let Some(caps) = re.captures(string) else {
            return Err("Could not parse regex.");
        };

        match &caps.name("cmd") {
            Some(cmd) => {
                match cmd.as_str() {
                    ".file" => {
                        return Ok(CommandType::File(caps["name"].to_string()));
                    }
                    ".image" => {
                        return Ok(CommandType::Image(caps["name"].to_string()));
                    }
                    ".username" => {
                        return Ok(CommandType::Username(caps["name"].to_string()));
                    }
                    _ => {
                        // this should never happen
                    }
                }
            }
            None => {}
        }

        match &caps.name("quit") {
            Some(_) => {
                return Ok(CommandType::Quit);
            }
            None => {}
        }

        match &caps.name("color") {
            Some(_) => {
                let r = u8::from_str_radix(&caps["r"], 10).unwrap();
                let g = u8::from_str_radix(&caps["g"], 10).unwrap();
                let b = u8::from_str_radix(&caps["b"], 10).unwrap();
                return Ok(CommandType::Color((r, g, b)));
            }
            None => {}
        }

        // there should not be any other option
        return Ok(CommandType::Text(caps["text"].to_string()));
    }
}

pub fn handle_send_message(mut stream: TcpStream, input: &str) -> Result<bool, Box<dyn Error>> {
    // create CommandType from String
    let command_type = CommandType::from_str(&input)?;

    // flag to determine if quit command is performed
    let mut quit = false;

    // convert CommandType to MessageType and possibly fill it's value
    let message_type = match command_type {
        // for CommandType::Text set only text
        CommandType::Text(text) => MessageType::Text(text),

        // for CommandType::File and CommandType::Image read specified file and send it as vector
        // for CommandType::File also parse file name
        CommandType::File(ref file_path) | CommandType::Image(ref file_path) => {
            let mut file = File::open(file_path)?;

            let mut buffer_send = vec![];
            file.read_to_end(&mut buffer_send)?;

            match command_type {
                CommandType::File(_) => {
                    let file_name = match Path::new(file_path.as_str()).file_name() {
                        Some(s) => match s.to_str() {
                            Some(s) => s.to_string(),
                            None => return Err("Could convert file name to String.".into()),
                        },
                        None => return Err("Could not parse file name.".into()),
                    };

                    MessageType::File(file_name, buffer_send)
                }
                CommandType::Image(_) => MessageType::Image(buffer_send),
                _ => {
                    return Err("This should never happen.".into());
                }
            }
        }

        // for CommandType::Quit set quit flag to true
        CommandType::Quit => {
            quit = true;
            MessageType::UserDisconnect()
        }

        // for CommandType::Username set new username
        CommandType::Username(new_username) => {
            MessageType::UserNameChange(new_username.to_string())
        }

        // for CommandType::Color set values for red, green and blue
        CommandType::Color((r, g, b)) => MessageType::UserColorChange(r, g, b),
    };

    // create message structure, user info is default because this informations fills only server
    let message = Message {
        message: message_type,
        user_info: UserInfo {
            ..Default::default()
        },
        datetime: SystemTime::now(),
    };

    // send message to server
    send_message(&mut stream, &message)?;

    // return true if quit command is performed
    if quit {
        return Ok(true);
    }

    // otherwise return false
    Ok(false)
}

pub fn handle_receive_message(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    // read message from server
    let message = read_message(&mut stream)?;

    // convert user's name color to Color enum
    let username_color = Color::Rgb {
        r: message.user_info.color.0,
        g: message.user_info.color.1,
        b: message.user_info.color.2,
    };

    // print output to command line and do other actions based on MessageType
    match message.message {
        // for MessageType::Text print user's name and message
        MessageType::Text(s) => {
            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            println!("> {}", s);
        }

        // for MessageType::File save file to ./files directory and print info about file to user
        MessageType::File(file_name, data) => {
            let mut my_file_name = "./files/".to_string();
            my_file_name = my_file_name + file_name.as_str();

            let mut file = File::create(my_file_name.clone())?;

            let _ = file.write_all(data.as_slice());

            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            println!(
                "> send you file '{}' (on your pc '{}').",
                file_name, my_file_name
            );
        }

        // for MessageType::Image save image to ./images directory and print image to command line
        MessageType::Image(data) => {
            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            println!(">");

            let mut my_file_name = "./images/".to_string();
            my_file_name = my_file_name + Utc::now().timestamp().to_string().as_str() + ".png";

            let img = Reader::new(Cursor::new(data))
                .with_guessed_format()?
                .decode()?;

            let _ = img.save_with_format(my_file_name, image::ImageFormat::Png);

            let conf = Config {
                absolute_offset: false,
                ..Default::default()
            };

            viuer::print(&img, &conf)?;
        }

        // for MessageType::UserNameChange print old and new name of user
        MessageType::UserNameChange(old_name) => {
            print!("User '");
            print_colored_string_to_stdout(old_name.as_str(), username_color)?;
            print!("' changed his name to '");
            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            println!("'.");
        }

        // for MessageType::UserConnect print name of the user who connected
        MessageType::UserConnect() => {
            print!("User '");
            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            println!("' connected.");
        }

        // for MessageType::UserDisconnect() print name of the user who disconnected
        MessageType::UserDisconnect() => {
            print!("User '");
            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            println!("' disconnected.");
        }

        // for MessageType::UserColorChange print old and new color of user's name
        MessageType::UserColorChange(or, og, ob) => {
            print!("User '");
            print_colored_string_to_stdout(message.user_info.username.as_str(), username_color)?;
            print!("' changed his color from ");
            print_colored_string_to_stdout(
                "■",
                Color::Rgb {
                    r: or,
                    g: og,
                    b: ob,
                },
            )?;
            print!(" to ");
            print_colored_string_to_stdout("■", username_color)?;
            println!(".");
        }
    }

    Ok(())
}

fn print_colored_string_to_stdout(username: &str, color: Color) -> Result<(), std::io::Error> {
    // print string with set color and then reset color to the original
    execute!(
        io::stdout(),
        SetForegroundColor(color),
        Print(username),
        ResetColor
    )
}
