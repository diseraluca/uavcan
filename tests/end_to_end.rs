// TODO: Remember to refactor the common setups of the tests

use heapless::consts::{U512, U64};
use proptest::collection::vec;
use proptest::prelude::*;
use uavcan::{
    rx::rx_network::RxNetwork,
    session_id::{
        session_kind::strategy::session_kind, NodeId, SessionKind, SubjectId, TransferPriority,
    },
    tx::{stream_transmitter::StreamTransmitter, transmitter::send},
    CLASSIC_MTU,
};

mod common;
use common::{can_frame::ClassicFrame, can_tx::TxRxGlue};

proptest! {
    #[test]
    fn receiving_the_frames_of_a_transmission_rebuilds_the_original_payload(payload in vec(proptest::num::u8::ANY, 1..100)) {
        let mut rx_network = RxNetwork::<ClassicFrame, U64, U512, CLASSIC_MTU>::new();
        let (rx_producer, mut rx_consumer) = rx_network.split();

        let mut transmitter = StreamTransmitter::<TxRxGlue<ClassicFrame, U64, U512, CLASSIC_MTU>, ClassicFrame, CLASSIC_MTU>::new(TxRxGlue{ rx_producer });

        let node_id = NodeId::new();
        send(
            &mut transmitter,
            &payload,
            SessionKind::Message {
                source_node_id: node_id,
                subject_id: SubjectId::new(),
            },
            TransferPriority::High,
        )
            .unwrap();

        let reconstructed_payload = rx_consumer.next().unwrap().payload;

        prop_assert_eq!(AsRef::<[u8]>::as_ref(&reconstructed_payload), &payload);

    }
}

proptest! {
    #[test]
    fn receiving_the_frames_of_a_transmission_rebuilds_the_original_session_kind(payload in vec(proptest::num::u8::ANY, 1..100), kind in session_kind()) {
        let mut rx_network = RxNetwork::<ClassicFrame, U64, U512, CLASSIC_MTU>::new();
        let (rx_producer, mut rx_consumer) = rx_network.split();

        let mut transmitter = StreamTransmitter::<TxRxGlue<ClassicFrame, U64, U512, CLASSIC_MTU>, ClassicFrame, CLASSIC_MTU>::new(TxRxGlue{ rx_producer });

        send(
            &mut transmitter,
            &payload,
            kind,
            TransferPriority::High,
        )
            .unwrap();

        prop_assert_eq!(rx_consumer.next().unwrap().kind, kind);

    }
}
