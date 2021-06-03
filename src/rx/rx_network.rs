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
    pub fn split<'a>(
        &'a mut self,
    ) -> (
        RxProducer<'a, Frame, Capacity, TransferCapacity, MTU>,
        RxConsumer<'a, Frame, Capacity, TransferCapacity, MTU>,
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
        // println!("Received frame {:?}", frame);
        match self
            .buildup
            .get_or_insert_with(|| Buildup::default())
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
