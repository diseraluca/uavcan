use uavcan::{CanFrame, CLASSIC_MTU};

#[derive(Debug)]
pub struct ClassicFrame {
    data: [u8; 8],
    id: u32,
    len: usize,
}

impl From<(u32, [u8; 8], usize)> for ClassicFrame {
    fn from((id, data, len): (u32, [u8; 8], usize)) -> Self {
        Self { data, id, len }
    }
}

impl CanFrame<CLASSIC_MTU> for ClassicFrame {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> (&[u8; CLASSIC_MTU], usize) {
        (&self.data, self.len)
    }
}
