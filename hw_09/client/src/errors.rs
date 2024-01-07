use std::fmt::{Display, Formatter};

use image::ImageError;
use thiserror::Error;
use viuer::ViuError;

use libs::errors::MessageError;

#[derive(Error, Debug)]
pub enum ReceiveMessageError {
    Server,
    Io(#[from] std::io::Error),
    Image(#[from] ImageError),
    ImageConsolePrint(#[from] ViuError),
    Message(#[from] MessageError),
}

impl Display for ReceiveMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "ReceiveMessageError")
    }
}

#[derive(Error, Debug)]
pub enum SendMessageError {
    Io(#[from] std::io::Error),
    MessageType(#[from] FromStrError),
    Internal(String),
    File(String),
    Message(#[from] MessageError),
}

impl Display for SendMessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "SendMessageError")
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum FromStrError {
    RegexCreate,
    RegexParse,
    StringToNumber,
    Internal(String),
}

impl Display for FromStrError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "FromStrError")
    }
}
