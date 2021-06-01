// TODO: Remember to deal with the fact that a 0 payload is unacceptable. Might
// be better to deal with this outside breakdown.

// TODO: A general refactoring of this must be done. Previously a
// misunderstanding made it so that we expected the CRC to always be on its
// frame even when enough space for it was available. This is now changed but
// the code was a bit tight and has now some replication that should be removed.

use core::{marker::PhantomData, slice::ChunksExact};

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

/// Describes the possible positions of the crc in a multi frame transfer.
///
/// During a multi frame transfer the crc may be positioned in three different
/// ways depending on the amount of data to send and the MTU of the transfer.
///
/// We call a crc `Isolated`, when it is sent as the only data ( excluded the
/// tail byte ) in last frame of the transfer.
///
/// We call a crc `HalfEmbedded`, when its LSB is positioned as the last
/// non-tail byte in the second to last frame that is sent and its MSB is
/// positioned as the only data ( excluded the tail byte ) of the last frame of
/// the transfer.
///
/// We call a crc `Embedded`, when it occupies the last two non-tail-bytes in
/// the last frame of the transfer.
#[derive(Debug)]
pub enum CRCKind {
    Embedded,
    HalfEmbedded,
    Isolated,
}

/// Identifies the position of the crc in a multi frame transfer.
///
/// `payload_remainder` represents the remainder of the division between the
/// length of the payload of the transfer and the MTU with the space for the
/// tail byte removed (MTU-1).
///
/// For example, if the MTU is 8, each frame will contain 7 bytes of data and a tail byte.
/// If a payload of length 9 is to be transferred, the first 7 bytes will be
/// positioned on a frame, and the remainder of two bytes will occupy the first two bytes of another frame.
///
/// If, for example, a payload, with length N that is a multiple of (MTU-1) is
/// to be sent, exactly (N/(MTU-1)) fully occupied frames will need to be made
/// to sent the whole payload.
///
/// Each multi frame transfer has to be ended with a two bytes crc of the payload data.
/// Those bytes may thus occupy three different positions:
///
/// They may be embedded into the last frame containing some of the payload
/// data. This happens when the difference between (MTU-1) and the remainder of
/// the payload is at least 2; that is, when at least two bytes of data are
/// available after filling the frame with the payload data.
///
/// The first byte may be embedded in the last frame containing some of the
/// payload data and the second byte into a frame of its own. This happens when
/// the difference between (MTU-1) and the remainder of the payload is exactly
/// 1; that is, there is at only on byte of data available in after filling the
/// frame with the payload data.
///
/// Both bytes are embedded on their own frame. This happens when there is no
/// remainder; that is, when there is no available space in the last frame
/// containing some of the payload data after filling it with the payload data.
///
/// This function returns an identifier describing which of the three case of
/// crc positioning should be used, based on the remainder that is provided.
///
/// See [CRCKind] for a description of the meaning of the identifier returned by [crc_kind].
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
            &self.crc.get_crc().to_be_bytes()[1..],
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

impl<Frame: CanFrame<MTU>, const MTU: usize> Iterator for Breakdown<'_, Frame, MTU> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    extern crate std;
    use std::format;

    use crate::CLASSIC_MTU;

    proptest! {
        #[test]
        fn the_crc_is_to_be_embedded_when_the_last_frame_of_the_payload_has_at_least_three_available_bytes(last_frame_data_len in 1..CLASSIC_MTU-3 ) {
            prop_assert!(matches!(crc_kind::<CLASSIC_MTU>(last_frame_data_len), CRCKind::Embedded))
        }
    }

    #[test]
    fn the_crc_is_to_be_half_embedded_when_the_last_frame_of_the_payload_has_exactly_two_available_bytes(
    ) {
        assert!(matches!(
            crc_kind::<CLASSIC_MTU>(CLASSIC_MTU - 2),
            CRCKind::HalfEmbedded
        ))
    }

    proptest! {
        #[test]
        fn the_crc_is_to_be_isolated_when_the_last_frame_of_the_payload_has_less_than_two_available_bytes(last_frame_data_len in CLASSIC_MTU-1..CLASSIC_MTU) {
            prop_assert!(matches!(crc_kind::<CLASSIC_MTU>(last_frame_data_len), CRCKind::Isolated))
        }
    }

    // TODO: This test is not actually meaningful considering the specification
    // of the kind of the crc as `depending on the available bytes in the last
    // frame of the payload.`
    // If the "last frame of the payload" has no payload data, that frame wouldn't exist at all.
    // This inconsistencies comes from the specific implementation of `crc_kind` as based on the
    // `remainder of the payload length divided by the amount of space available for each frame`,
    // which is needed to slot it more easily into the way that breakdown is built ( specifically the use of chunks ).
    // Either reform the tests to be a specification of this implemented behavior or change the implementation to
    // respect the specification of the tests which should render this test meaningless.
    #[test]
    fn the_crc_is_to_be_isolated_when_the_last_frame_of_the_payload_has_a_length_of_0() {
        assert!(matches!(crc_kind::<CLASSIC_MTU>(0), CRCKind::Isolated))
    }
}
