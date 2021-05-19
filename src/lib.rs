pub mod rx;
pub mod session_id;
pub mod tail_byte;
pub mod tx;

pub const CLASSIC_MTU: usize = 8;
pub const EXTENDED_MTU: usize = 64;

pub trait CanFrame<const MTU: usize>: From<(u32, [u8; MTU], usize)> + core::fmt::Debug {
    fn id(&self) -> u32;
    fn payload(&self) -> (&[u8; MTU], usize);
}
