use std::{
    fs::File,
    io::{self, Cursor, Read, Write},
    path::Path,
    str::FromStr,
};

use anyhow::Result;
use chrono::Utc;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use image::io::Reader;
use viuer::Config;

use commands::CommandType;
use errors::{ReceiveMessageError, SendMessageError};
use libs::{
    message::{Message, MessageType},
    receiver::MessageReceiver,
    sender::MessageSender,
};

pub mod args;
pub mod commands;
pub mod errors;

pub async fn handle_send_message(
    sender: &mut MessageSender,
    input: &str,
) -> Result<bool, SendMessageError> {
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
                            None => {
                                return Err(SendMessageError::File(
                                    "Could convert file name to String.".to_string(),
                                ))
                            }
                        },
                        None => {
                            return Err(SendMessageError::File(
                                "Could not parse file name.".to_string(),
                            ))
                        }
                    };

                    MessageType::File(file_name, buffer_send)
                }
                CommandType::Image(_) => MessageType::Image(buffer_send),
                _ => {
                    return Err(SendMessageError::Internal(
                        "This should never happen.".to_string(),
                    ));
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

    // create message structure, user info and date time are default because this informations fills only server
    let message = Message::from(message_type);

    // send message to server
    sender.send_message(&message).await?;

    // return true if quit command is performed
    if quit {
        return Ok(true);
    }

    // otherwise return false
    Ok(false)
}

pub async fn handle_receive_message(
    receiver: &mut MessageReceiver,
) -> Result<(), ReceiveMessageError> {
    // read message from server
    let message = receiver.receive_message().await?;

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

        // for MessageType::RecoverableError print error message
        MessageType::RecoverableError(error) => {
            print_colored_string_to_stdout(&error, Color::Red)?;
            println!();
        }

        // for MessageType::UnrecoverableError print error message and return ClientError
        MessageType::UnrecoverableError(error) => {
            print_colored_string_to_stdout(&error, Color::Red)?;
            println!();
            return Err(ReceiveMessageError::Server);
        }

        // for MessageType::LoginRequest nothing should be done, this message is only client -> server
        MessageType::LoginRequest(..) => {}

        // for MessageType::LoginResponse determine if login was successful and print message
        MessageType::LoginResponse(success) => {
            match success {
                Some(_) => {
                    print_colored_string_to_stdout("Login was successful.", Color::Green)?;
                }
                None => {
                    print_colored_string_to_stdout("Login failed.", Color::Red)?;
                }
            }

            println!();
        }

        // for MessageType::RegisterRequest nothing should be done, this message is only client -> server
        MessageType::RegisterRequest(..) => {}

        // for MessageType::LoginResponse determine if registration was successful and print message
        MessageType::RegisterResponse(success) => {
            match success {
                Some(_) => {
                    print_colored_string_to_stdout("Registration was successful.", Color::Green)?;
                }
                None => {
                    print_colored_string_to_stdout("Registration failed.", Color::Red)?;
                }
            }

            println!();
        }

        // for MessageType::OldMessagesRequest nothing should be done, this message is only client -> server
        MessageType::OldMessagesRequest() => {}

        // for MessageType::OldMessagesResponse print all old messages send by server
        MessageType::OldMessagesResponse(messages) => {
            for message in messages {
                // convert user's name color to Color enum
                let username_color = Color::Rgb {
                    r: message.1.color.0,
                    g: message.1.color.1,
                    b: message.1.color.2,
                };

                print_colored_string_to_stdout(message.1.username.as_str(), username_color)?;
                println!("> {}", message.0);
            }
        }
    }

    Ok(())
}

pub fn print_colored_string_to_stdout(string: &str, color: Color) -> Result<(), std::io::Error> {
    // print string with set color and then reset color to the original
    execute!(
        io::stdout(),
        SetForegroundColor(color),
        Print(string),
        ResetColor
    )
}
