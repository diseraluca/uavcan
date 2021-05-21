use super::{message::MessageSessionId, service::ServiceSessionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionId {
    Message(MessageSessionId),
    Rpc(ServiceSessionId),
}

impl SessionId {
    pub fn is_valid(&self) -> bool {
        match self {
            SessionId::Message(message) => message.is_valid(),
            SessionId::Rpc(service) => service.is_valid(),
        }
    }
}

impl From<u32> for SessionId {
    fn from(value: u32) -> Self {
        if ((value >> 25) & 1) == 0 {
            SessionId::Message(MessageSessionId::from(value))
        } else {
            SessionId::Rpc(ServiceSessionId::from(value))
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn building_a_session_id_from_a_number_with_the_25th_bit_not_set_provides_a_message() {
//         let id = SessionId::try_from(0u32).unwrap();

//         assert!(matches!(id, SessionId::Message(_)))
//     }
// }
