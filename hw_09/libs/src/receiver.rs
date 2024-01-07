use std::sync::Arc;

use tokio::{io::AsyncReadExt, net::tcp::OwnedReadHalf, sync::Mutex};

use crate::{errors::MessageError, message::Message};

#[derive(Clone)]
pub struct MessageReceiver {
    stream: Arc<Mutex<OwnedReadHalf>>,
}

impl MessageReceiver {
    pub fn from_owned_read_half(stream: OwnedReadHalf) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub async fn receive_message(&mut self) -> Result<Message, MessageError> {
        let mut len = [0; 4];

        let mut stream = self.stream.lock().await;

        stream.read_exact(&mut len).await?;

        let len = u32::from_be_bytes(len) as usize;

        let mut buffer = vec![0; len];

        stream.read_exact(&mut buffer).await?;

        Ok(Message::deserialize(&buffer)?)
    }
}
