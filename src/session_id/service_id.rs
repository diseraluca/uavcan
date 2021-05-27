use super::error::OutOfBounds;
use core::convert::TryFrom;
use modular_bitfield::prelude::*;

#[bitfield(bits = 9)]
#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
pub struct ServiceId {
    #[skip(getters)]
    id: B9,
}

impl TryFrom<u16> for ServiceId {
    type Error = OutOfBounds;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        (value < 512)
            .then(|| ServiceId::new().with_id(value))
            .ok_or(OutOfBounds {})
    }
}

#[cfg(test)]
pub mod strategy {
    use super::*;
    use proptest::prop_compose;

    prop_compose! {
        pub fn service_id()(id in 0..512u16) -> ServiceId {
            ServiceId::try_from(id).unwrap()
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
        fn a_service_id_is_a_positive_integer_in_the_closed_interval_0_511(id in 0..512u16) {
            prop_assert!(ServiceId::try_from(id).is_ok())
        }
    }

    proptest! {
        #[test]
        fn building_a_service_id_outside_its_defined_range_is_an_error(id in 512..u16::MAX) {
            prop_assert!(ServiceId::try_from(id).is_err())
        }
    }
}
