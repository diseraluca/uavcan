use super::{node_id::NodeId, service_id::ServiceId, TransferPriority};
use modular_bitfield::prelude::*;

#[bitfield]
#[repr(u32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ServiceSessionId {
    #[bits = 7]
    pub source_node_id: NodeId,
    #[bits = 7]
    pub destination_node_id: NodeId,
    #[bits = 9]
    pub service_id: ServiceId,
    reserved23: B1,
    pub is_request: bool,
    #[skip(getters)]
    is_service: bool,
    #[bits = 3]
    #[skip(getters)]
    priority: TransferPriority,
    #[skip]
    __: B3,
}

impl ServiceSessionId {
    pub fn request_as_u32(
        source_node_id: NodeId,
        destination_node_id: NodeId,
        service_id: ServiceId,
        priority: TransferPriority,
    ) -> u32 {
        ServiceSessionId::new()
            .with_source_node_id(source_node_id)
            .with_destination_node_id(destination_node_id)
            .with_service_id(service_id)
            .with_is_service(true)
            .with_is_request(true)
            .with_priority(priority)
            .into()
    }

    pub fn response_as_u32(
        source_node_id: NodeId,
        destination_node_id: NodeId,
        service_id: ServiceId,
        priority: TransferPriority,
    ) -> u32 {
        ServiceSessionId::new()
            .with_source_node_id(source_node_id)
            .with_destination_node_id(destination_node_id)
            .with_service_id(service_id)
            .with_is_service(true)
            .with_is_request(false)
            .with_priority(priority)
            .into()
    }

    pub fn is_valid(&self) -> bool {
        self.reserved23() == 0
    }
}

#[cfg(test)]
pub mod strategy {
    use super::super::node_id::strategy::node_id;
    use super::super::service_id::strategy::service_id;
    use super::super::transfer_priority::strategy::transfer_priority;
    use super::*;
    use proptest::prop_compose;

    prop_compose! {
        pub fn request_session_id_as_u32()(source_node_id in node_id(), destination_node_id in node_id(), service_id in service_id(), transfer_priority in transfer_priority()) -> u32 {
            ServiceSessionId::request_as_u32(source_node_id, destination_node_id, service_id, transfer_priority)
        }
    }

    prop_compose! {
        pub fn response_session_id_as_u32()(source_node_id in node_id(), destination_node_id in node_id(), service_id in service_id(), transfer_priority in transfer_priority()) -> u32 {
            ServiceSessionId::response_as_u32(source_node_id, destination_node_id, service_id, transfer_priority)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::node_id::strategy::node_id;
    use super::super::service_id::strategy::service_id;
    use super::super::transfer_priority::strategy::transfer_priority;
    use super::strategy::{request_session_id_as_u32, response_session_id_as_u32};
    use super::*;
    use core::convert::TryFrom;
    use proptest::prelude::*;

    extern crate std;
    use std::format;

    proptest! {
        #[test]
        fn a_service_session_id_has_its_27th_to_29th_bit_represent_the_transfer_priority(transfer_priority in transfer_priority()) {
            let (source_node_id, destination_node_id, service_id) = (NodeId::try_from(4).unwrap(), NodeId::try_from(7).unwrap(), ServiceId::try_from(10).unwrap());
            let request_id = ServiceSessionId::request_as_u32(source_node_id, destination_node_id, service_id, transfer_priority);
            let response_id = ServiceSessionId::response_as_u32(source_node_id, destination_node_id, service_id, transfer_priority);

            prop_assert_eq!(TransferPriority::try_from(((request_id >> 26) & 7)as u8).unwrap(), transfer_priority);
            prop_assert_eq!(TransferPriority::try_from(((response_id >> 26) & 7)as u8).unwrap(), transfer_priority);
        }
    }

    proptest! {
        #[test]
        fn a_service_session_id_has_its_26th_bit_set(request_id in request_session_id_as_u32(), response_id in response_session_id_as_u32()) {
            prop_assert_eq!((request_id >> 25) & 1, 1);
            prop_assert_eq!((response_id >> 25) & 1, 1);
        }
    }

    proptest! {
        #[test]
        fn a_response_service_session_id_has_its_25_bit_not_set(response_id in response_session_id_as_u32()) {
            prop_assert_eq!((response_id >> 24) & 1, 0);
        }
    }

    proptest! {
        #[test]
        fn a_request_service_session_id_has_its_25_bit_set(request_id in request_session_id_as_u32()) {
            prop_assert_eq!((request_id >> 24) & 1, 1);
        }
    }

    proptest! {
        #[test]
        fn a_service_session_id_has_its_24th_bit_not_set(request_id in request_session_id_as_u32(), response_id in response_session_id_as_u32()) {
            prop_assert_eq!((request_id >> 23) & 1, 0);
            prop_assert_eq!((response_id >> 23) & 1, 0);
        }
    }

    proptest! {
        #[test]
        fn a_service_session_id_has_its_15th_to_23rd_bit_represent_the_service_id(service_id in service_id()) {
            let (source_node_id, destination_node_id, transfer_priority) = (NodeId::try_from(4).unwrap(), NodeId::try_from(7).unwrap(), TransferPriority::Immediate);
            let request_id = ServiceSessionId::request_as_u32(source_node_id, destination_node_id, service_id, transfer_priority);
            let response_id = ServiceSessionId::response_as_u32(source_node_id, destination_node_id, service_id, transfer_priority);

            prop_assert_eq!(ServiceId::try_from(((request_id >> 14) & 511)as u16).unwrap(), service_id);
            prop_assert_eq!(ServiceId::try_from(((response_id >> 14) & 511)as u16).unwrap(), service_id);
        }
    }

    proptest! {
         #[test]
         fn a_service_session_id_has_its_8th_to_14th_bit_represent_the_service_id(destination_node_id in node_id()) {
             let (source_node_id, service_id, transfer_priority) = (NodeId::try_from(4).unwrap(), ServiceId::try_from(7).unwrap(), TransferPriority::Immediate);
             let request_id = ServiceSessionId::request_as_u32(source_node_id, destination_node_id, service_id, transfer_priority);
             let response_id = ServiceSessionId::response_as_u32(source_node_id, destination_node_id, service_id, transfer_priority);

             prop_assert_eq!(NodeId::try_from(((request_id >> 7) & 127)as u8).unwrap(), destination_node_id);
             prop_assert_eq!(NodeId::try_from(((response_id >> 7) & 127)as u8).unwrap(), destination_node_id);
         }
    }
}
