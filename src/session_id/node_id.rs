use super::error::OutOfBounds;
use core::convert::TryFrom;
use modular_bitfield::prelude::*;

#[bitfield(bits = 7)]
#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
pub struct NodeId {
    #[skip(getters)]
    id: B7,
}

impl TryFrom<u8> for NodeId {
    type Error = OutOfBounds;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        (value < 128)
            .then(|| NodeId::new().with_id(value))
            .ok_or(OutOfBounds {})
    }
}

#[cfg(test)]
pub mod strategy {
    use super::*;
    use proptest::prop_compose;

    prop_compose! {
        pub fn node_id()(id in 0..128u8) -> NodeId {
            NodeId::try_from(id).unwrap()
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
        fn a_node_id_is_a_positive_integer_in_the_closed_interval_0_127(id in 0..128u8) {
            prop_assert!(NodeId::try_from(id).is_ok())
        }
    }

    proptest! {
        #[test]
        fn building_a_node_id_outside_its_defined_range_is_an_error(id in 128..u8::MAX) {
            prop_assert!(NodeId::try_from(id).is_err())
        }
    }
}
