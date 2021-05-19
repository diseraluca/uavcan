use socketcan::{CANFrame, CANSocket};
use std::convert::TryFrom;
use uavcan::session_id::{NodeId, SessionKind, SubjectId, TransferPriority};
use uavcan::tx::{
    stream_transmitter::{CanWriter, StreamTransmitter},
    transmitter::send,
};
use uavcan::{CanFrame, CLASSIC_MTU};

#[derive(Debug)]
struct ClassicFrame {
    data: [u8; 8],
    id: u32,
}

impl From<(u32, [u8; 8], usize)> for ClassicFrame {
    fn from((id, data, len): (u32, [u8; 8], usize)) -> Self {
        Self { data, id }
    }
}

impl CanFrame<CLASSIC_MTU> for ClassicFrame {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> (&[u8; CLASSIC_MTU], usize) {
        (&self.data, 8)
    }
}

struct CanTx(CANSocket);

impl CanWriter<ClassicFrame, CLASSIC_MTU> for CanTx {
    type Error = std::io::Error;

    fn write_frame(&mut self, frame: ClassicFrame) -> Result<(), Self::Error> {
        self.0
            .write_frame(&CANFrame::new(frame.id(), frame.payload().0, false, false).unwrap())
    }
}

fn main() {
    let payload = [23u8, 42, 10];

    let tx_socket = CANSocket::open("vcan0").unwrap();
    let mut transmitter =
        StreamTransmitter::<CanTx, ClassicFrame, CLASSIC_MTU>::new(CanTx(tx_socket));

    send(
        &mut transmitter,
        &payload,
        SessionKind::Message {
            source_node_id: NodeId::try_from(23).unwrap(),
            subject_id: SubjectId::try_from(45).unwrap(),
        },
        TransferPriority::High,
    )
    .unwrap();
}
