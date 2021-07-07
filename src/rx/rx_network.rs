use core::{convert::TryInto, marker::PhantomData};

use super::{
    buildup::{self, Buildup, BuildupState},
    transfer::Transfer,
};
use crate::CanFrame;
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::ArrayLength;

#[derive(Debug)]
pub enum RxError<Frame: CanFrame<MTU>, const MTU: usize> {
    OutOfSpace,
    ZeroLengthFrame,
    BuildupError(buildup::Error<Frame, MTU>),
}

pub struct RxConsumer<
    'a,
    Frame: CanFrame<MTU>,
    Capacity: ArrayLength<Transfer<TransferCapacity>>,
    TransferCapacity: ArrayLength<u8>,
    const MTU: usize,
> {
    consumer: Consumer<'a, Transfer<TransferCapacity>, Capacity>,
    _frame_marker: PhantomData<Frame>,
}

pub struct RxProducer<
    'a,
    Frame: CanFrame<MTU>,
    Capacity: ArrayLength<Transfer<TransferCapacity>>,
    TransferCapacity: ArrayLength<u8>,
    const MTU: usize,
> {
    producer: Producer<'a, Transfer<TransferCapacity>, Capacity>,
    buildup: Option<Buildup<Frame, TransferCapacity, MTU>>,
}

pub struct RxNetwork<
    Frame: CanFrame<MTU>,
    Capacity: ArrayLength<Transfer<TransferCapacity>>,
    TransferCapacity: ArrayLength<u8>,
    const MTU: usize,
> {
    queue: Queue<Transfer<TransferCapacity>, Capacity>,
    _frame_marker: PhantomData<Frame>,
}

impl<
        Frame: CanFrame<MTU>,
        Capacity: ArrayLength<Transfer<TransferCapacity>>,
        TransferCapacity: ArrayLength<u8>,
        const MTU: usize,
    > Default for RxNetwork<Frame, Capacity, TransferCapacity, MTU>
{
    fn default() -> Self {
        Self {
            queue: Queue::new(),
            _frame_marker: PhantomData,
        }
    }
}

impl<
        Frame: CanFrame<MTU>,
        Capacity: ArrayLength<Transfer<TransferCapacity>>,
        TransferCapacity: ArrayLength<u8>,
        const MTU: usize,
    > RxNetwork<Frame, Capacity, TransferCapacity, MTU>
{
    pub fn split(
        &mut self,
    ) -> (
        RxProducer<Frame, Capacity, TransferCapacity, MTU>,
        RxConsumer<Frame, Capacity, TransferCapacity, MTU>,
    ) {
        let (producer, consumer) = self.queue.split();

        (
            RxProducer {
                producer,
                buildup: None,
            },
            RxConsumer {
                consumer,
                _frame_marker: PhantomData,
            },
        )
    }
}

impl<
        Frame: CanFrame<MTU>,
        Capacity: ArrayLength<Transfer<TransferCapacity>>,
        TransferCapacity: ArrayLength<u8>,
        const MTU: usize,
    > Iterator for RxConsumer<'_, Frame, Capacity, TransferCapacity, MTU>
{
    type Item = Transfer<TransferCapacity>;

    fn next(&mut self) -> Option<Self::Item> {
        self.consumer.dequeue()
    }
}

impl<
        Frame: CanFrame<MTU>,
        Capacity: ArrayLength<Transfer<TransferCapacity>>,
        TransferCapacity: ArrayLength<u8>,
        const MTU: usize,
    > RxProducer<'_, Frame, Capacity, TransferCapacity, MTU>
{
    pub fn receive(&mut self, frame: Frame) -> Result<(), RxError<Frame, MTU>> {
        if let (_, 0) = frame.payload() {
            return Err(RxError::ZeroLengthFrame);
        }

        match self
            .buildup
            .get_or_insert_with(Buildup::default)
            .push(frame)
        {
            Ok(BuildupState::Closed) => self
                .producer
                .enqueue(self.buildup.take().unwrap().try_into().unwrap())
                .map_err(|_| RxError::OutOfSpace),
            Err(err) => {
                self.buildup.take();

                Err(RxError::BuildupError(err))
            }
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::tests::ClassicFrame;
    use crate::CLASSIC_MTU;
    use heapless::consts::{U512, U64};

    #[test]
    fn receiving_a_frame_with_no_data_results_in_an_error() {
        let mut network = RxNetwork::<ClassicFrame, U64, U512, CLASSIC_MTU>::default();
        let (mut producer, _) = network.split();
        let empty_payload: [u8; 8] = [0; 8];

        assert!(producer
            .receive(ClassicFrame::from((0, empty_payload, 0)))
            .is_err());
    }
}
