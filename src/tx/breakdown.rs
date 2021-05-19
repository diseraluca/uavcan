// TODO: Remember to deal with the fact that a 0 payload is unacceptable. Might
// be better to deal with this outside breakdown.

// TODO: A general refactoring of this must be done. Previously a
// misunderstanding made it so that we expected the CRC to always be on its
// frame even when enough space for it was available. This is now changed but
// the code was a bit tight and has now some replication that should be removed.

use core::{marker::PhantomData, panic, slice::ChunksExact};

use crc_any::CRCu16;

use crate::{
    tail_byte::{TailByte, TransferId},
    CanFrame,
};

#[derive(Debug, Copy, Clone)]
pub enum BreakdownKind {
    SingleFrame,
    MultiFrame(usize),
}

pub fn breakdown_kind_for_payload<const MTU: usize>(payload: &[u8]) -> BreakdownKind {
    let payload_size = payload.len();

    if payload_size < MTU {
        BreakdownKind::SingleFrame
    } else {
        let frame_count_without_crc_frame = payload_size / MTU;
        let frame_count = frame_count_without_crc_frame + 1;

        BreakdownKind::MultiFrame(frame_count)
    }
}

#[derive(Debug)]
pub enum CRCKind {
    Embedded,
    HalfEmbedded,
    Isolated,
}

fn crc_kind<const MTU: usize>(payload_remainder: usize) -> CRCKind {
    // TODO: Cannot match here because of pattern restriction for constants and
    // expressions. There might be a way to do this remember to check when
    // possible.

    let remaining_space = MTU - payload_remainder - 1;
    if remaining_space == 0 || payload_remainder == 0 {
        CRCKind::Isolated
    } else if remaining_space == 1 {
        CRCKind::HalfEmbedded
    } else if (2..=MTU).contains(&remaining_space) {
        CRCKind::Embedded
    } else {
        core::panic!(
            "CRCKind is unknown. This should not happen. \
            Are you sure that this function was called with \
            the length of the last data frame in the transfer?"
        );
    }
}

fn build_frame<Frame: CanFrame<MTU>, const MTU: usize>(
    can_id: u32,
    data: &[u8],
    tail_byte: TailByte,
) -> Frame {
    let mut payload = [0u8; MTU];
    payload[..data.len()].copy_from_slice(data);
    payload[data.len()] = tail_byte.into_u8();

    Frame::from((can_id, payload, data.len() + 1))
}

#[derive(Debug)]
pub enum BreakdownState {
    SingleFrame,
    MultiFrame,
    MultiFrameHalfCRC,
    Closed,
}

impl From<BreakdownKind> for BreakdownState {
    fn from(kind: BreakdownKind) -> Self {
        match kind {
            BreakdownKind::SingleFrame => BreakdownState::SingleFrame,
            BreakdownKind::MultiFrame(_) => BreakdownState::MultiFrame,
        }
    }
}

pub struct Breakdown<'a, Frame: CanFrame<MTU>, const MTU: usize> {
    payload: ChunksExact<'a, u8>,
    state: BreakdownState,
    crc: CRCu16,
    can_id: u32,
    tail_byte: TailByte,
    _frame_marker: PhantomData<Frame>,
}

impl<'a, Frame: CanFrame<MTU>, const MTU: usize> Breakdown<'a, Frame, MTU> {
    pub fn new(payload: &'a [u8], can_id: u32) -> Self {
        let breakdown_kind = breakdown_kind_for_payload::<MTU>(payload);
        let tail_byte = match breakdown_kind {
            BreakdownKind::SingleFrame => TailByte::single_frame(TransferId::new()),
            BreakdownKind::MultiFrame(_) => TailByte::start_of_multi_frame(TransferId::new()),
        };

        Self {
            payload: payload.chunks_exact(MTU - 1),
            state: BreakdownState::from(breakdown_kind),
            crc: CRCu16::crc16ccitt_false(),
            can_id,
            tail_byte,
            _frame_marker: PhantomData,
        }
    }

    // TODO: this need to change considering the new change to CRC positioning
    pub fn frames_count(&self) -> usize {
        self.payload.size_hint().0 + 1
    }

    pub fn transfer_id(&self) -> TransferId {
        self.tail_byte.get_transfer_id()
    }

    pub fn build_single_frame(&mut self) -> Frame {
        self.state = BreakdownState::Closed;

        let payload = self.payload.next().unwrap_or(self.payload.remainder());
        build_frame(self.can_id, payload, self.tail_byte)
    }

    fn build_multi_frame(&mut self) -> Frame {
        match self.payload.next() {
            Some(data) => {
                self.crc.digest(data);

                let frame = build_frame(self.can_id, data, self.tail_byte);
                self.tail_byte.advance();

                frame
            }
            None => {
                let data = self.payload.remainder();
                self.crc.digest(data);

                match crc_kind::<MTU>(data.len()) {
                    CRCKind::Embedded => {
                        let crc_bytes = self.crc.get_crc().to_be_bytes();

                        let mut data_with_crc = [0u8; MTU];
                        data_with_crc[..data.len()].copy_from_slice(data);
                        data_with_crc[data.len()..data.len() + 2].copy_from_slice(&crc_bytes);

                        self.state = BreakdownState::Closed;

                        build_frame(
                            self.can_id,
                            &data_with_crc[..data.len() + 2],
                            self.tail_byte.end_of_multi_transfer(),
                        )
                    }
                    CRCKind::HalfEmbedded => {
                        let tail_byte = self.tail_byte;
                        let crc_bytes = self.crc.get_crc().to_be_bytes();

                        let mut data_with_crc = [0u8; MTU];
                        data_with_crc[..data.len()].copy_from_slice(data);
                        data_with_crc[data.len()] = crc_bytes[0];

                        self.state = BreakdownState::MultiFrameHalfCRC;
                        self.tail_byte.advance();

                        build_frame(self.can_id, &data_with_crc[..data.len() + 1], tail_byte)
                    }
                    CRCKind::Isolated => self.build_crc(),
                }
            }
        }
    }

    fn build_half_crc(&mut self) -> Frame {
        self.state = BreakdownState::Closed;

        self.tail_byte = self.tail_byte.end_of_multi_transfer();

        build_frame(
            self.can_id,
            &self.crc.get_crc().to_be_bytes()[..1],
            self.tail_byte,
        )
    }

    fn build_crc(&mut self) -> Frame {
        self.state = BreakdownState::Closed;

        self.tail_byte = self.tail_byte.end_of_multi_transfer();

        build_frame(
            self.can_id,
            &self.crc.get_crc().to_be_bytes(),
            self.tail_byte,
        )
    }
}

impl<'a, Frame: CanFrame<MTU>, const MTU: usize> Iterator for Breakdown<'a, Frame, MTU> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            BreakdownState::SingleFrame => Some(self.build_single_frame()),
            BreakdownState::MultiFrame => Some(self.build_multi_frame()),
            BreakdownState::MultiFrameHalfCRC => Some(self.build_half_crc()),
            BreakdownState::Closed => None,
        }
    }
}
