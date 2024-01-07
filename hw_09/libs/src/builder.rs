use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{errors::MessageError, receiver::MessageReceiver, sender::MessageSender};

pub struct MessageReceiverSenderBuilder {
    message_receiver: MessageReceiver,
    message_sender: MessageSender,
}

impl MessageReceiverSenderBuilder {
    pub async fn from_socket_addr<T: ToSocketAddrs>(
        addr: T,
    ) -> Result<MessageReceiverSenderBuilder, MessageError> {
        let stream = TcpStream::connect(addr).await?;

        MessageReceiverSenderBuilder::from_tcp_stream(stream)
    }

    pub fn from_tcp_stream(
        stream: TcpStream,
    ) -> Result<MessageReceiverSenderBuilder, MessageError> {
        let (read, write) = stream.into_split();

        Ok(MessageReceiverSenderBuilder {
            message_receiver: MessageReceiver::from_owned_read_half(read),
            message_sender: MessageSender::from_owned_write_half(write),
        })
    }

    pub fn message_receiver(&self) -> MessageReceiver {
        self.message_receiver.clone()
    }

    pub fn message_sender(&self) -> MessageSender {
        self.message_sender.clone()
    }
}
