use std::io::Write;
use std::net::{TcpStream, ToSocketAddrs};

use crate::errors::MessageError;
use crate::message::Message;
use crate::receiver::MessageReceiver;

pub struct MessageSender {
    stream: TcpStream,
}

impl MessageSender {
    pub fn new<T: ToSocketAddrs>(addr: T) -> Result<Self, MessageError> {
        Ok(Self {
            stream: TcpStream::connect(addr)?,
        })
    }

    pub fn from_tcp_stream(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn send_message(&mut self, message: &Message) -> Result<(), MessageError> {
        let serialized = message.serialize()?;

        let len = serialized.len() as u32;
        self.stream.write(&len.to_be_bytes())?;

        self.stream.write_all(serialized.as_slice())?;

        Ok(())
    }
}

impl MessageReceiver {
    pub fn from_message_sender(sender: &MessageSender) -> Result<MessageReceiver, MessageError> {
        Ok(MessageReceiver::from_tcp_stream(sender.stream.try_clone()?))
    }
}
