use super::transfer_id::TransferId;
use modular_bitfield::prelude::*;

#[derive(Debug)]
pub enum PayloadKind {
    StartOfMultiFrame,
    EndOfMultiFrame,
    SingleFrame,
    MiddleOfMultiFrame,
}

#[bitfield]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TailByte {
    #[bits = 5]
    transfer_id: TransferId,
    toggle: B1,
    is_end_of_transfer: bool,
    is_start_of_transfer: bool,
}

impl TailByte {
    pub fn single_frame(transfer_id: TransferId) -> Self {
        Self::new()
            .with_is_start_of_transfer(true)
            .with_is_end_of_transfer(true)
            .with_toggle(1)
            .with_transfer_id(transfer_id)
    }

    pub fn start_of_multi_frame(transfer_id: TransferId) -> TailByte {
        Self::new()
            .with_is_start_of_transfer(true)
            .with_is_end_of_transfer(false)
            .with_toggle(1)
            .with_transfer_id(transfer_id)
    }

    pub fn into_u8(self) -> u8 {
        u8::from_le_bytes(self.into_bytes())
    }

    // TODO: hack fix remember to do correctly
    pub fn split_from<const SIZE: usize>(payload: (&[u8; SIZE], usize)) -> (&[u8], TailByte) {
        // if payload[SIZE - 1] != 0 {
        //     (&payload[..SIZE - 1], Self::from_bytes([payload[SIZE - 1]]))
        // } else {
        let (payload, len) = payload;
        // let mut tail_index: usize = SIZE - 1;
        // for index in 0..SIZE - 1 {
        //     if payload[SIZE - 1 - index] != 0 {
        //         tail_index = SIZE - 1 - index;
        //         break;
        //     }
        // }

        (&payload[..len - 1], Self::from_bytes([payload[len - 1]]))
        // }
    }

    pub fn from_u8(byte: u8) -> Self {
        Self::from_bytes([byte])
    }

    pub fn payload_kind(&self) -> PayloadKind {
        match (self.is_start_of_transfer(), self.is_end_of_transfer()) {
            (true, true) => PayloadKind::SingleFrame,
            (true, false) => PayloadKind::StartOfMultiFrame,
            (false, true) => PayloadKind::EndOfMultiFrame,
            (false, false) => PayloadKind::MiddleOfMultiFrame,
        }
    }

    pub fn get_transfer_id(&self) -> TransferId {
        self.transfer_id()
    }

    // TODO: Review the way in which this works. Depends on breakdown and
    // buildup as they currently need to advance manually before closing the
    // transfer.
    pub fn end_of_multi_transfer(&self) -> TailByte {
        let mut byte = self.clone();

        // byte.set_toggle(byte.toggle() ^ 1);
        byte.set_is_start_of_transfer(false);
        byte.set_is_end_of_transfer(true);

        // let mut transfer_id = byte.transfer_id();
        // transfer_id.advance();

        // byte.set_transfer_id(transfer_id);

        byte
    }

    pub fn advance(&mut self) {
        self.set_toggle(self.toggle() ^ 1);
        self.set_is_start_of_transfer(false);
        self.set_is_end_of_transfer(false);

        // let mut transfer_id = self.transfer_id();
        // transfer_id.advance();

        // self.set_transfer_id(transfer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::convert::TryFrom;

    #[test]
    fn a_single_frame_tail_byte_is_a_start_of_transfer() {
        let tail_byte = TailByte::single_frame(TransferId::try_from(0).unwrap());

        assert!(tail_byte.is_start_of_transfer())
    }

    #[test]
    fn a_single_frame_tail_byte_is_an_end_of_transfer() {
        let tail_byte = TailByte::single_frame(TransferId::try_from(0).unwrap());

        assert!(tail_byte.is_end_of_transfer())
    }

    #[test]
    fn a_single_frame_tail_byte_has_a_toggle_of_1() {
        let tail_byte = TailByte::single_frame(TransferId::try_from(0).unwrap());

        assert_eq!(tail_byte.toggle(), 1);
    }

    #[test]
    fn a_single_frame_tail_byte_has_the_provided_transfer_id() {
        let transfer_id = TransferId::try_from(3).unwrap();
        let tail_byte = TailByte::single_frame(transfer_id);

        assert_eq!(tail_byte.transfer_id(), transfer_id);
    }

    #[test]
    fn a_start_of_transfer_multi_frame_tail_byte_is_a_start_of_transfer() {
        let tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());

        assert!(tail_byte.is_start_of_transfer())
    }

    #[test]
    fn a_start_of_transfer_multi_frame_tail_byte_is_not_an_end_of_transfer() {
        let tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());

        assert!(!tail_byte.is_end_of_transfer())
    }

    #[test]
    fn a_start_of_transfer_multi_frame_tail_byte_has_a_toggle_of_1() {
        let tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());

        assert_eq!(tail_byte.toggle(), 1)
    }

    #[test]
    fn a_start_of_transfer_multi_frame_tail_byte_has_the_provided_transfer_id() {
        let transfer_id = TransferId::try_from(3).unwrap();
        let tail_byte = TailByte::start_of_multi_frame(transfer_id);

        assert_eq!(tail_byte.transfer_id(), transfer_id);
    }

    #[test]
    fn an_end_of_transfer_multi_frame_tail_byte_is_not_a_start_of_transfer() {
        let tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap())
            .end_of_multi_transfer();

        assert!(!tail_byte.is_start_of_transfer())
    }

    #[test]
    fn an_end_of_transfer_multi_frame_tail_byte_is_an_end_of_transfer() {
        let tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap())
            .end_of_multi_transfer();

        assert!(tail_byte.is_end_of_transfer())
    }

    #[test]
    fn an_end_of_transfer_multi_frame_tail_byte_preserves_the_toggle_of_the_original_tail_byte() {
        let tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());
        let end_of_transfer_tail_byte = tail_byte.end_of_multi_transfer();

        assert_eq!(end_of_transfer_tail_byte.toggle(), tail_byte.toggle());
    }

    #[test]
    fn an_end_of_transfer_multi_frame_tail_byte_preserves_the_transfer_id_of_the_original_byte() {
        let transfer_id = TransferId::try_from(4).unwrap();
        let tail_byte = TailByte::start_of_multi_frame(transfer_id).end_of_multi_transfer();

        assert_eq!(tail_byte.transfer_id(), transfer_id);
    }

    #[test]
    fn the_successor_of_a_tail_byte_is_not_a_start_of_transfer() {
        let mut tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());
        tail_byte.advance();

        assert!(!tail_byte.is_start_of_transfer());
    }

    #[test]
    fn the_successor_of_a_tail_byte_is_not_an_end_of_transfer() {
        let mut tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());
        tail_byte.advance();

        assert!(!tail_byte.is_end_of_transfer());
    }

    #[test]
    fn the_successor_of_a_tail_byte_inverts_the_toggle_of_the_original_tail_byte() {
        let mut tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());
        let original_toggle = tail_byte.toggle();

        tail_byte.advance();

        assert_eq!(tail_byte.toggle(), original_toggle ^ 1);
    }

    #[test]
    fn the_successor_of_a_tail_byte_has_the_same_transfer_id_as_the_original_tail_byte(
    ) {
        let mut tail_byte = TailByte::start_of_multi_frame(TransferId::try_from(0).unwrap());
        let original_transfer_id = tail_byte.transfer_id();

        tail_byte.advance();

        assert_eq!(tail_byte.transfer_id(), original_transfer_id);
    }
}
