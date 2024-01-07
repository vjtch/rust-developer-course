use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::errors::MessageError;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub message: MessageType,
    pub user_info: UserInfo,
    pub datetime: SystemTime,
}

impl Message {
    pub fn serialize(&self) -> Result<Vec<u8>, MessageError> {
        Ok(bincode::serialize(self)?)
    }

    pub fn deserialize(bytes: &[u8]) -> Result<Self, MessageError> {
        Ok(bincode::deserialize(&bytes)?)
    }
}

impl From<MessageType> for Message {
    fn from(message_type: MessageType) -> Self {
        Message {
            message: message_type,
            user_info: UserInfo::default(),
            datetime: SystemTime::now(),
        }
    }
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
    RecoverableError(String),
    UnrecoverableError(String),
    LoginRequest(String, String),
    LoginResponse(Option<UserInfo>),
    RegisterRequest(String, String, u8, u8, u8),
    RegisterResponse(Option<UserInfo>),
    OldMessagesRequest(),
    OldMessagesResponse(Vec<(String, UserInfo)>),
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct UserInfo {
    pub id: i32,
    pub username: String,
    pub color: (u8, u8, u8),
}
