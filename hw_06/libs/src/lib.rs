use std::error::Error;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub message: MessageType,
    pub user_info: UserInfo,
    pub datetime: SystemTime,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum MessageType {
    Text(String),
    Image(Vec<u8>),
    File(String, Vec<u8>),
    UserConnect(),
    UserDisconnect(),
    UserNameChange(String),
    UserColorChange(u8, u8, u8),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserInfo {
    pub username: String,
    pub color: (u8, u8, u8),
}

fn serialize_message(message: &Message) -> Result<Vec<u8>, Box<bincode::ErrorKind>> {
    bincode::serialize(message)
}

fn deserialize_message(bytes: &[u8]) -> Result<Message, Box<bincode::ErrorKind>> {
    bincode::deserialize(&bytes)
}

pub fn send_message(stream: &mut TcpStream, message: &Message) -> Result<(), Box<dyn Error>> {
    let serialized = serialize_message(message)?;

    let len = serialized.len() as u32;
    stream.write(&len.to_be_bytes())?;

    stream.write_all(serialized.as_slice())?;

    Ok(())
}

pub fn read_message(stream: &mut TcpStream) -> Result<Message, Box<dyn Error>> {
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;

    let mut len = [0; 4];
    stream.read_exact(&mut len)?;

    let len = u32::from_be_bytes(len) as usize;

    let mut buffer = vec![0; len];
    stream.read_exact(&mut buffer)?;

    Ok(deserialize_message(&buffer)?)
}

pub fn remove_new_line(string: &mut String) {
    *string = string
        .trim_end_matches('\n')
        .trim_end_matches('\r')
        .to_string();
}
