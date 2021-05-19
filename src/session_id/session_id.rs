use super::{error::InvalidRepresentation, message::MessageSessionId, service::ServiceSessionId};
use core::convert::TryFrom;

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

impl TryFrom<u32> for SessionId {
    type Error = InvalidRepresentation;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        let is_service = (value >> 25) & 1;
        match is_service {
            0 => Ok(SessionId::Message(MessageSessionId::from(value))),
            0 => Ok(SessionId::Rpc(ServiceSessionId::from(value))),
            _ => panic!("Error in the bit pattern when converting a u32 to a session id. This shouldn't happen, a logic bug may be in place.")
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
