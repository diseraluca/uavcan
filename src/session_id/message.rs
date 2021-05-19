use super::{node_id::NodeId, subject_id::SubjectId, transfer_priority::TransferPriority};
use modular_bitfield::prelude::*;

#[bitfield]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MessageSessionId {
    #[bits = 7]
    pub source_node_id: NodeId,
    reserved7: B1,
    #[bits = 13]
    pub subject_id: SubjectId,
    reserved21: B1,
    reserved22: B1,
    reserved23: B1,
    is_anonymous: bool,
    pub is_service: bool,
    #[bits = 3]
    priority: TransferPriority,
    #[skip]
    __: B3,
}

impl MessageSessionId {
    pub fn as_u32(
        source_node_id: NodeId,
        subject_id: SubjectId,
        priority: TransferPriority,
    ) -> u32 {
        MessageSessionId::new()
            .with_source_node_id(source_node_id)
            .with_subject_id(subject_id)
            .with_reserved21(1)
            .with_reserved22(1)
            .with_is_anonymous(false)
            .with_is_service(false)
            .with_priority(priority)
            .into()
    }

    pub fn is_valid(&self) -> bool {
        self.reserved23() == 0 && self.reserved7() == 0
    }
}

#[cfg(any(debug_assertions, test))]
pub mod strategy {
    use super::super::node_id::strategy::node_id;
    use super::super::subject_id::strategy::subject_id;
    use super::super::transfer_priority::strategy::transfer_priority;
    use super::*;
    use proptest::prop_compose;

    prop_compose! {
        pub fn message_session_id_as_u32()(source_node_id in node_id(), subject_id in subject_id(), transfer_priority in transfer_priority()) -> u32 {
            MessageSessionId::as_u32(source_node_id, subject_id, transfer_priority)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::node_id::strategy::node_id;
    use super::super::subject_id::strategy::subject_id;
    use super::super::transfer_priority::strategy::transfer_priority;
    use super::strategy::message_session_id_as_u32;
    use super::*;
    use core::convert::TryFrom;
    use proptest::prelude::*;

    extern crate std;
    use std::format;

    proptest! {
        #[test]
        fn a_message_session_id_has_its_27th_to_29th_bit_represent_the_transfer_priority(transfer_priority in transfer_priority()) {
            let id = MessageSessionId::as_u32(NodeId::try_from(4).unwrap(), SubjectId::try_from(8).unwrap(), transfer_priority);
            prop_assert_eq!(TransferPriority::try_from(((id >> 26) & 7)as u8).unwrap(), transfer_priority);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_26th_bit_not_set(id in message_session_id_as_u32()) {
            prop_assert_eq!((id >> 25) & 1, 0);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_24th_bit_not_set(id in message_session_id_as_u32()) {
            prop_assert_eq!((id >> 23) & 1, 0);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_23th_bit_set(id in message_session_id_as_u32()) {
            prop_assert_eq!((id >> 22) & 1, 1);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_22th_bit_set(id in message_session_id_as_u32()) {
            prop_assert_eq!((id >> 22) & 1, 1);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_9th_to_21st_bit_represent_the_subject_id(subject_id in subject_id()) {
            let id = MessageSessionId::as_u32(NodeId::try_from(4).unwrap(), subject_id, TransferPriority::Immediate);
            prop_assert_eq!(SubjectId::try_from(((id >> 8) & 8191) as u16).unwrap(), subject_id);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_8th_bit_not_set(id in message_session_id_as_u32()) {
            prop_assert_eq!((id >> 7) & 1, 0);
        }
    }

    proptest! {
        #[test]
        fn a_message_session_id_has_its_first_7_bits_represent_the_node_id(node_id in node_id()) {
            let id = MessageSessionId::as_u32(node_id, SubjectId::try_from(4).unwrap(), TransferPriority::Immediate);
            prop_assert_eq!(NodeId::try_from((id & 127) as u8).unwrap(), node_id);
        }
    }

    proptest! {
        #[test]
        fn building_a_message_session_id_as_u32_and_converting_it_back_to_a_message_session_id_preserves_source_node_id(source_node_id in node_id()) {
            let subject_id = SubjectId::new();
            let transfer_priority = TransferPriority::High;

            let as_u32 = MessageSessionId::as_u32(source_node_id, subject_id, transfer_priority);
            let message_session_id = MessageSessionId::from(as_u32);

            prop_assert_eq!(message_session_id.source_node_id(), source_node_id);
        }
    }

    proptest! {
        #[test]
        fn building_a_message_session_id_as_u32_and_converting_it_back_to_a_message_session_id_preserves_subject_id(subject_id in subject_id()) {
            let source_node_id = NodeId::new();
            let transfer_priority = TransferPriority::High;

            let as_u32 = MessageSessionId::as_u32(source_node_id, subject_id, transfer_priority);
            let message_session_id = MessageSessionId::from(as_u32);

            prop_assert_eq!(message_session_id.subject_id(), subject_id);
        }
    }

    proptest! {
        #[test]
        fn building_a_message_session_id_as_u32_and_converting_it_back_to_a_message_session_id_preserves_transfer_priority(transfer_priority in transfer_priority()) {
            let source_node_id = NodeId::new();
            let subject_id = SubjectId::new();

            let as_u32 = MessageSessionId::as_u32(source_node_id, subject_id, transfer_priority);
            let message_session_id = MessageSessionId::from(as_u32);

            prop_assert_eq!(message_session_id.priority(), transfer_priority);
        }
    }
}
