#![allow(incomplete_features)]
#![deny(
    noop_method_call,
    single_use_lifetimes,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_import_braces,
    unused_lifetimes,
    warnings
)]
#![no_std]
#![feature(cfg_eval)]

pub mod rx;
pub mod session_id;
pub mod tail_byte;
pub mod tx;

pub const CLASSIC_MTU: usize = 8;
pub const EXTENDED_MTU: usize = 64;

pub trait CanFrame<const MTU: usize>: From<(u32, [u8; MTU], usize)> + core::fmt::Debug {
    fn id(&self) -> u32;
    fn payload(&self) -> (&[u8; MTU], usize);
}

#[cfg(test)]
mod tests {
    // TODO: There is quite a few setup code in the preamble and some duplicated
    // code in the tests. Evaluate if there is a better way to modularize it.

    use super::*;
    use super::{
        rx::{
            rx_network::{RxError, RxNetwork, RxProducer},
            transfer::Transfer,
        },
        session_id::{
            session_kind::strategy::session_kind, NodeId, SessionKind, SubjectId, TransferPriority,
        },
        tx::{
            stream_transmitter::{CanWriter, StreamTransmitter},
            transmitter::send,
        },
        CLASSIC_MTU,
    };

    use heapless::{
        consts::{U512, U64},
        ArrayLength,
    };

    use proptest::collection::vec;
    use proptest::prelude::*;

    extern crate std;
    use std::format;

    #[derive(Debug)]
    pub(super) struct ClassicFrame {
        data: [u8; 8],
        id: u32,
        len: usize,
    }

    impl From<(u32, [u8; 8], usize)> for ClassicFrame {
        fn from((id, data, len): (u32, [u8; 8], usize)) -> Self {
            Self { data, id, len }
        }
    }

    impl CanFrame<CLASSIC_MTU> for ClassicFrame {
        fn id(&self) -> u32 {
            self.id
        }

        fn payload(&self) -> (&[u8; CLASSIC_MTU], usize) {
            (&self.data, self.len)
        }
    }

    pub(super) struct TxRxGlue<
        'a,
        Frame: CanFrame<MTU>,
        Capacity: ArrayLength<Transfer<TransferCapacity>>,
        TransferCapacity: ArrayLength<u8>,
        const MTU: usize,
    > {
        pub(super) rx_producer: RxProducer<'a, Frame, Capacity, TransferCapacity, MTU>,
    }

    impl<
            Frame: CanFrame<MTU>,
            Capacity: ArrayLength<Transfer<TransferCapacity>>,
            TransferCapacity: ArrayLength<u8>,
            const MTU: usize,
        > CanWriter<Frame, MTU> for TxRxGlue<'_, Frame, Capacity, TransferCapacity, MTU>
    {
        type Error = RxError<Frame, MTU>;

        fn write_frame(&mut self, frame: Frame) -> Result<(), Self::Error> {
            self.rx_producer.receive(frame)
        }
    }

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
}
