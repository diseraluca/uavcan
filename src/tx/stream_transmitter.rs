use core::marker::PhantomData;

use super::transmitter::Transmitter;
use crate::CanFrame;

pub trait CanWriter<Frame: CanFrame<MTU>, const MTU: usize> {
    type Error;

    fn write_frame(&mut self, frame: Frame) -> Result<(), Self::Error>;
}

pub struct StreamTransmitter<Writer: CanWriter<Frame, MTU>, Frame: CanFrame<MTU>, const MTU: usize>
{
    writer: Writer,
    _marker: PhantomData<Frame>,
}

impl<Writer: CanWriter<Frame, MTU>, Frame: CanFrame<MTU>, const MTU: usize>
    StreamTransmitter<Writer, Frame, MTU>
{
    pub fn new(writer: Writer) -> Self {
        Self {
            writer,
            _marker: PhantomData,
        }
    }
}

impl<Frame: CanFrame<MTU>, Writer: CanWriter<Frame, MTU>, const MTU: usize> Transmitter<Frame, MTU>
    for StreamTransmitter<Writer, Frame, MTU>
{
    type Error = Writer::Error;

    fn transmit(&mut self, frame: Frame) -> Result<(), Self::Error> {
        self.writer.write_frame(frame)?;

        Ok(())
    }
}
