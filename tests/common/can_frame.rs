use uavcan::{CanFrame, CLASSIC_MTU};

#[derive(Debug)]
pub struct ClassicFrame {
    data: [u8; 8],
    id: u32,
}

impl From<(u32, [u8; 8])> for ClassicFrame {
    fn from((id, data): (u32, [u8; 8])) -> Self {
        Self { data, id }
    }
}

impl CanFrame<CLASSIC_MTU> for ClassicFrame {
    fn id(&self) -> u32 {
        self.id
    }

    fn payload(&self) -> &[u8; CLASSIC_MTU] {
        &self.data
    }
}
