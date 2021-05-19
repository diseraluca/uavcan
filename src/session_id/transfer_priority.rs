use modular_bitfield::prelude::*;

#[derive(BitfieldSpecifier, Debug, Copy, Clone, PartialEq, Eq)]
#[bits = 3]
pub enum TransferPriority {
    Exceptional = 0,
    Immediate = 1,
    Fast = 2,
    High = 3,
    Nominal = 4,
    Low = 5,
    Slow = 6,
    Optional = 7,
}

// We need to be able to convert an integer value to TransferPriority to be able
// to express the specification of entities that encode it such as SessionId.
// Nontheless, this type of conversion should not be used, for now, in the rest
// of the codebase, as TransferPriority is expected to be created by the user
// trough the enum interface only. As such, the implementation of TryFrom is
// locked behind the test flag.
#[cfg(test)]
impl core::convert::TryFrom<u8> for TransferPriority {
    type Error = super::error::InvalidRepresentation;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(TransferPriority::Exceptional),
            1 => Ok(TransferPriority::Immediate),
            2 => Ok(TransferPriority::Fast),
            3 => Ok(TransferPriority::High),
            4 => Ok(TransferPriority::Nominal),
            5 => Ok(TransferPriority::Low),
            6 => Ok(TransferPriority::Slow),
            7 => Ok(TransferPriority::Optional),
            _ => Err(super::error::InvalidRepresentation {}),
        }
    }
}

#[cfg(any(debug_assertions, test))]
pub mod strategy {
    use super::*;
    use proptest::{
        prop_oneof,
        strategy::{Just, Strategy},
    };

    pub fn transfer_priority() -> impl Strategy<Value = TransferPriority> {
        prop_oneof![
            Just(TransferPriority::Exceptional),
            Just(TransferPriority::Immediate),
            Just(TransferPriority::Fast),
            Just(TransferPriority::High),
            Just(TransferPriority::Nominal),
            Just(TransferPriority::Low),
            Just(TransferPriority::Slow),
            Just(TransferPriority::Optional),
        ]
    }
}
