use heapless::consts::*;
use rand::RngCore;
use socketcan::{CANFrame, CANSocket};
use uavcan::rx::rx_network::{RxConsumer, RxNetwork, RxProducer};
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

impl From<(u32, [u8; 8])> for ClassicFrame {
    fn from((id, data): (u32, [u8; 8])) -> Self {
        Self { data, id }
    }
}

impl From<CANFrame> for ClassicFrame {
    fn from(frame: CANFrame) -> Self {
        let (id, original_data) = (frame.id(), frame.data());

        let mut data = [0u8; 8];
        data[..original_data.len()].copy_from_slice(original_data);

        Self::from((id, data))
    }
}

impl CanFrame<CLASSIC_MTU> for ClassicFrame {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> &[u8; CLASSIC_MTU] {
        &self.data
    }
}

struct CanTx(CANSocket);

impl CanWriter<ClassicFrame, CLASSIC_MTU> for CanTx {
    type Error = std::io::Error;

    fn write_frame(&mut self, frame: ClassicFrame) -> Result<(), Self::Error> {
        self.0
            .write_frame(&CANFrame::new(frame.id(), frame.payload(), false, false).unwrap())
    }
}

fn transmit(
    transmitter: &mut StreamTransmitter<CanTx, ClassicFrame, CLASSIC_MTU>,
    node_id: NodeId,
) -> () {
    println!("Building random payload for transmission.");
    let mut payload = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut payload);
    println!("Random payload for transmission was built.");

    println!("Sending {:?}.", payload);
    send(
        transmitter,
        &payload,
        SessionKind::Message {
            source_node_id: node_id,
            subject_id: SubjectId::new(),
        },
        TransferPriority::High,
    )
    .unwrap();
    println!("Payload sent");
}

fn receive<'a>(
    rx_socket: &CANSocket,
    receiver: &mut RxProducer<'a, ClassicFrame, U64, U512, CLASSIC_MTU>,
) -> () {
    println!("Looking for frames from socket.");
    while let Ok(frame) = rx_socket.read_frame() {
        println!("Found frame {:?}.", frame);
        println!("Storing frame for later.");
        receiver.receive(ClassicFrame::from(frame)).unwrap();
        println!("Frame stored for later.");
    }
}

fn process<'a>(receiver: &mut RxConsumer<'a, ClassicFrame, U64, U512, CLASSIC_MTU>) -> () {
    println!("Looking for stored transfers.");
    while let Some(transfer) = receiver.next() {
        println!("Found transfer {:?}", transfer);
    }
}

fn main() {
    println!("Opening Sockets.");
    let tx_socket = CANSocket::open("vcan0").unwrap();
    let rx_socket = CANSocket::open("vcan0").unwrap();
    rx_socket
        .set_read_timeout(std::time::Duration::from_millis(500))
        .unwrap();
    println!("Socket opened.");

    println!("Initializing transmitter.");
    let mut transmitter =
        StreamTransmitter::<CanTx, ClassicFrame, CLASSIC_MTU>::new(CanTx(tx_socket));
    println!("Transmitter initialized.");

    println!("Initializing receiver network.");
    let mut rx_network = RxNetwork::<ClassicFrame, U64, U512, CLASSIC_MTU>::new();
    let (mut rx_producer, mut rx_consumer) = rx_network.split();
    println!("Receiver network initialized.");

    println!("Building node id.");
    let node_id = NodeId::new();
    println!("Node id built.");

    println!("Starting the loop.");
    loop {
        transmit(&mut transmitter, node_id);
        receive(&rx_socket, &mut rx_producer);
        process(&mut rx_consumer);
    }
}
