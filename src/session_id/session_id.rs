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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_can_id_with_the_26th_bit_not_set_is_a_message() {
        let id = SessionId::from(0u32);

        assert!(matches!(id, SessionId::Message(_)))
    }

    #[test]
    fn a_can_id_with_the_26th_bit_set_is_an_rpc() {
        let id = SessionId::from(1u32 << 25);

        assert!(matches!(id, SessionId::Rpc(_)))
    }
}
