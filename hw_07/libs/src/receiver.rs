use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::errors::MessageError;
use crate::message::Message;
use crate::sender::MessageSender;

pub struct MessageReceiver {
    stream: TcpStream,
}

impl MessageReceiver {
    pub fn new<T: ToSocketAddrs>(addr: T) -> Result<Self, MessageError> {
        Ok(Self {
            stream: TcpStream::connect(addr)?,
        })
    }

    pub fn from_tcp_stream(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn receive_message(&mut self) -> Result<Message, MessageError> {
        self.stream
            .set_read_timeout(Some(Duration::from_millis(100)))?;

        let mut len = [0; 4];
        self.stream.read_exact(&mut len)?;

        let len = u32::from_be_bytes(len) as usize;

        let mut buffer = vec![0; len];
        self.stream.read_exact(&mut buffer)?;

        Ok(Message::deserialize(&buffer)?)
    }
}

impl MessageSender {
    pub fn from_message_receiver(
        receiver: &MessageReceiver,
    ) -> Result<MessageSender, MessageError> {
        Ok(MessageSender::from_tcp_stream(receiver.stream.try_clone()?))
    }
}
