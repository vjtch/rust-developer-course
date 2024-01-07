use std::sync::Arc;

use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::Mutex};

use crate::{errors::MessageError, message::Message};

#[derive(Clone)]
pub struct MessageSender {
    stream: Arc<Mutex<OwnedWriteHalf>>,
}

impl MessageSender {
    pub fn from_owned_write_half(stream: OwnedWriteHalf) -> Self {
        Self {
            stream: Arc::new(Mutex::new(stream)),
        }
    }

    pub async fn send_message(&mut self, message: &Message) -> Result<(), MessageError> {
        let serialized = message.serialize()?;
        let len = serialized.len() as u32;

        let mut stream = self.stream.lock().await;

        stream.write(&len.to_be_bytes()).await?;
        stream.write_all(serialized.as_slice()).await?;

        Ok(())
    }
}
