use core::convert::TryFrom;
use modular_bitfield::prelude::*;

#[derive(Debug)]
pub struct OutOfBounds {}

#[bitfield(filled = false)]
#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
pub struct TransferId {
    transfer_id: B5,
}

// TODO: Change advance and the code that uses it to work with returned next
// values instead of internal mutability.
// TODO: Consider generalizing the concept of TransferId instead of hardcoding the uavcan/can modulo 32 version.
impl TransferId {
    pub fn difference(&self, other: TransferId) -> u8 {
        let mut difference = (other.transfer_id() as i16) - (self.transfer_id() as i16);
        if difference < 0 {
            difference += 32;
        }

        difference as u8
    }

    pub fn advance(&mut self) {
        let mut transfer_id: u8 = self.transfer_id();
        if transfer_id == 31 {
            transfer_id = 0;
        } else {
            transfer_id += 1;
        }

        self.set_transfer_id(transfer_id);
    }
}

impl TryFrom<u8> for TransferId {
    type Error = OutOfBounds;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (value < 32)
            .then(|| TransferId::new().with_transfer_id(value))
            .ok_or(OutOfBounds {})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    extern crate std;
    use std::format;

    proptest! {
        #[test]
        fn a_transfer_id_is_a_positive_integer_in_the_closed_interval_0_31(id in 0..32u8) {
            prop_assert!(TransferId::try_from(id).is_ok())
        }
    }

    proptest! {
        #[test]
        fn building_a_transfer_id_outside_its_defined_range_is_an_error(id in 32..u8::MAX) {
            prop_assert!(TransferId::try_from(id).is_err())
        }
    }

    proptest! {
        #[test]
        fn the_successor_of_transfer_id_x_is_transfer_id_x_plus_1_for_x_less_than_31(x in 0..31u8) {
            let mut transfer_id = TransferId::try_from(x).unwrap();
            transfer_id.advance();

            prop_assert_eq!(transfer_id, TransferId::try_from(x + 1).unwrap())
        }
    }

    #[test]
    fn the_successor_of_transfer_id_31_is_transfer_id_0() {
        let mut transfer_id = TransferId::try_from(31).unwrap();
        transfer_id.advance();

        assert_eq!(transfer_id, TransferId::try_from(0).unwrap())
    }

    proptest! {
        #[test]
        fn the_difference_between_transfer_id_x_and_transfer_id_y_is_the_number_of_successors_between_x_and_y(x in 0..32u8, y in 0..32u8) {
            let target = TransferId::try_from(y).unwrap();

            let successors = std::iter::successors(
                Some((0u8, TransferId::try_from(x).unwrap())),
                |(n, mut source)| { source.advance(); Some((n + 1, source))}
            ).find(|(_, source)| *source == target).unwrap().0;

            prop_assert_eq!(TransferId::try_from(x).unwrap().difference(target), successors);
        }
    }
}
