use super::error::OutOfBounds;
use core::convert::TryFrom;
use modular_bitfield::prelude::*;

#[bitfield(bits = 13)]
#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
pub struct SubjectId {
    #[skip(getters)]
    id: B13,
}

impl TryFrom<u16> for SubjectId {
    type Error = OutOfBounds;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        (value < 8192)
            .then(|| SubjectId::new().with_id(value))
            .ok_or(OutOfBounds {})
    }
}

#[cfg(test)]
pub mod strategy {
    use super::*;
    use proptest::prop_compose;

    prop_compose! {
        pub fn subject_id()(id in 0..8192u16) -> SubjectId {
            SubjectId::try_from(id).unwrap()
        }
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
        fn a_subject_id_is_a_positive_integer_in_the_closed_interval_0_8192(id in 0..8192u16) {
            prop_assert!(SubjectId::try_from(id).is_ok())
        }
    }

    proptest! {
        #[test]
        fn building_a_subject_id_outside_its_defined_range_is_an_error(id in 8192..u16::MAX) {
            prop_assert!(SubjectId::try_from(id).is_err())
        }
    }
}
