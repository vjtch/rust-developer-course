use std::fmt::{Display, Formatter};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum MessageError {
    Io(#[from] std::io::Error),
    DeserializeSerialize(#[from] Box<bincode::ErrorKind>),
}

impl Display for MessageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "MessageError")
    }
}
