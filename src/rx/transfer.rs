use crate::session_id::SessionKind;
use heapless::{ArrayLength, Vec};

#[derive(Debug)]
pub struct Transfer<Capacity: ArrayLength<u8>> {
    pub payload: Vec<u8, Capacity>,
    pub kind: SessionKind,
}

impl<Capacity: ArrayLength<u8>> Transfer<Capacity> {
    pub fn new(payload: Vec<u8, Capacity>, kind: SessionKind) -> Self {
        Self { payload, kind }
    }
}
