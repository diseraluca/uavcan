use core::{convert::TryInto, marker::PhantomData};
use crc_any::CRCu16;
use heapless::{ArrayLength, Vec};

use crate::{
    session_id::{MessageSessionId, SessionId, SessionKind},
    tail_byte::{PayloadKind, TailByte, TransferId},
    CanFrame,
};

use super::transfer::Transfer;

#[derive(Debug)]
pub enum Error<Frame: CanFrame<MTU>, const MTU: usize> {
    CorruptedId,
    MissingFrames(u8),
    OutOfSpace,
    CannotAcceptNewFrames(Frame),
    WrongTypeOfFrame(BuildupState, PayloadKind, Frame),
    CorruptedTailByte(TailByte),
    WrongCRC(u16, u16),
}

#[derive(Debug)]
pub struct NotReady {}

#[derive(Debug, Clone, Copy)]
pub enum BuildupState {
    Initializing,
    MultiFrame,
    Closed,
    Errored,
}

pub enum BuildupKind {
    Unknown,
    Message,
    Request,
    Response,
}

pub struct Buildup<Frame: CanFrame<MTU>, Capacity: ArrayLength<u8>, const MTU: usize> {
    payload: Vec<u8, Capacity>,
    session_id: SessionId,
    state: BuildupState,
    tail_byte: TailByte,
    _frame_marker: PhantomData<Frame>,
}

impl<Frame: CanFrame<MTU>, Capacity: ArrayLength<u8>, const MTU: usize> Default
    for Buildup<Frame, Capacity, MTU>
{
    fn default() -> Self {
        Self {
            payload: Vec::new(),
            // TODO: Apart from documenting this the unintuitiveness of this
            // temporary value consider removing it entirely. Option is
            // problably a good way here.
            session_id: SessionId::Message(MessageSessionId::new()),
            state: BuildupState::Initializing,
            tail_byte: TailByte::new(),
            _frame_marker: PhantomData,
        }
    }
}

impl<Frame: CanFrame<MTU>, Capacity: ArrayLength<u8>, const MTU: usize>
    Buildup<Frame, Capacity, MTU>
{
    pub fn push(&mut self, frame: Frame) -> Result<BuildupState, Error<Frame, MTU>> {
        let session_id = SessionId::from(frame.id());
        session_id
            .is_valid()
            .then(|| ())
            .ok_or(Error::CorruptedId)?;

        self.advance(frame, session_id)
            .map_err(|e| {
                self.state = BuildupState::Errored;
                e
            })
            .map(|state| {
                self.state = state;
                state
            })
    }

    pub fn advance(
        &mut self,
        frame: Frame,
        session_id: SessionId,
    ) -> Result<BuildupState, Error<Frame, MTU>> {
        let (data, tail_byte) = TailByte::split_from(frame.payload());
        let payload_kind = tail_byte.payload_kind();

        match (self.state, payload_kind) {
            (BuildupState::Initializing, PayloadKind::StartOfMultiFrame) => {
                self.populate_first_frame(data, session_id)?;

                self.tail_byte = tail_byte;

                Ok(BuildupState::MultiFrame)
            }
            (BuildupState::Initializing, PayloadKind::SingleFrame) => {
                self.ensure_no_missing_frames(tail_byte.get_transfer_id())?;
                self.populate_first_frame(data, session_id)?;

                Ok(BuildupState::Closed)
            }
            (BuildupState::MultiFrame, PayloadKind::MiddleOfMultiFrame) => {
                self.tail_byte.advance();

                self.ensure_multiframe_integrity(session_id, tail_byte)?;

                self.save_payload(data)?;

                Ok(BuildupState::MultiFrame)
            }
            (BuildupState::MultiFrame, PayloadKind::EndOfMultiFrame) => {
                self.tail_byte.advance();
                self.tail_byte = self.tail_byte.end_of_multi_transfer();

                self.ensure_multiframe_integrity(session_id, tail_byte)?;

                let mut crc_bytes = [0u8; 2];
                match data.len() {
                    1 => {
                        crc_bytes[0] = self.payload.pop().unwrap();
                        crc_bytes[1] = data[0];
                    }
                    2 => {
                        crc_bytes.copy_from_slice(&data[..2]);
                    }
                    n => {
                        self.save_payload(&data[..n - 2])?;
                        crc_bytes.copy_from_slice(&data[n - 2..n]);
                    }
                };

                let crc = u16::from_be_bytes(crc_bytes);
                self.ensure_payload_integrity(crc)?;

                Ok(BuildupState::Closed)
            }
            (BuildupState::Closed, _) => Err(Error::CannotAcceptNewFrames(frame)),
            (BuildupState::Errored, _) => Err(Error::CannotAcceptNewFrames(frame)),
            (state, payload_kind) => Err(Error::WrongTypeOfFrame(state, payload_kind, frame)),
        }
    }

    fn save_payload(&mut self, payload: &[u8]) -> Result<(), Error<Frame, MTU>> {
        self.payload
            .extend_from_slice(payload)
            .map_err(|_| Error::OutOfSpace)
    }

    fn populate_first_frame(
        &mut self,
        payload: &[u8],
        session_id: SessionId,
    ) -> Result<(), Error<Frame, MTU>> {
        self.session_id = session_id;
        self.save_payload(payload)
    }

    fn ensure_multiframe_integrity(
        &self,
        session_id: SessionId,
        tail_byte: TailByte,
    ) -> Result<(), Error<Frame, MTU>> {
        self.ensure_no_missing_frames(tail_byte.get_transfer_id())?;
        self.ensure_tail_byte(tail_byte)?;
        self.ensure_session_id(session_id)
    }

    fn ensure_no_missing_frames(&self, transfer_id: TransferId) -> Result<(), Error<Frame, MTU>> {
        (self.tail_byte.get_transfer_id() == transfer_id)
            .then(|| ())
            .ok_or(Error::MissingFrames(
                self.tail_byte.get_transfer_id().difference(transfer_id),
            ))
    }

    fn ensure_tail_byte(&self, tail_byte: TailByte) -> Result<(), Error<Frame, MTU>> {
        (self.tail_byte == tail_byte)
            .then(|| ())
            .ok_or(Error::CorruptedTailByte(tail_byte))
    }

    fn ensure_session_id(&self, session_id: SessionId) -> Result<(), Error<Frame, MTU>> {
        (self.session_id == session_id)
            .then(|| ())
            .ok_or(Error::CorruptedId)
    }

    fn ensure_payload_integrity(&self, crc: u16) -> Result<(), Error<Frame, MTU>> {
        let mut own_crc = CRCu16::crc16ccitt_false();
        own_crc.digest(&self.payload);

        (own_crc.get_crc() == crc)
            .then(|| ())
            .ok_or(Error::WrongCRC(own_crc.get_crc(), crc))
    }
}

impl<Frame: CanFrame<MTU>, Capacity: ArrayLength<u8>, const MTU: usize> TryInto<Transfer<Capacity>>
    for Buildup<Frame, Capacity, MTU>
{
    type Error = NotReady;

    fn try_into(self) -> Result<Transfer<Capacity>, Self::Error> {
        match self.state {
            BuildupState::Closed => Ok(Transfer::new(
                self.payload,
                SessionKind::from(self.session_id),
            )),
            _ => Err(NotReady {}),
        }
    }
}
