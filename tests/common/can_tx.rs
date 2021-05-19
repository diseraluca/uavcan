use heapless::ArrayLength;
use uavcan::{
    rx::{
        rx_network::{RxError, RxProducer},
        transfer::Transfer,
    },
    tx::stream_transmitter::CanWriter,
    CanFrame,
};

pub struct TxRxGlue<
    'a,
    Frame: CanFrame<MTU>,
    Capacity: ArrayLength<Transfer<TransferCapacity>>,
    TransferCapacity: ArrayLength<u8>,
    const MTU: usize,
> {
    pub rx_producer: RxProducer<'a, Frame, Capacity, TransferCapacity, MTU>,
}

impl<
        'a,
        Frame: CanFrame<MTU>,
        Capacity: ArrayLength<Transfer<TransferCapacity>>,
        TransferCapacity: ArrayLength<u8>,
        const MTU: usize,
    > CanWriter<Frame, MTU> for TxRxGlue<'a, Frame, Capacity, TransferCapacity, MTU>
{
    type Error = RxError<Frame, MTU>;

    fn write_frame(&mut self, frame: Frame) -> Result<(), Self::Error> {
        self.rx_producer.receive(frame)
    }
}
