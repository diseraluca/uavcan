use super::breakdown::Breakdown;
use crate::session_id::TransferPriority;
use crate::{
    session_id::{can_id_for_session_kind, SessionKind},
    CanFrame,
};

pub trait Transmitter<Frame: CanFrame<MTU>, const MTU: usize> {
    type Error;

    fn transmit(&mut self, frame: Frame) -> Result<(), Self::Error>;

    fn ensure_available_space(&self, _frames_count: usize) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn send<T: Transmitter<Frame, MTU>, Frame: CanFrame<MTU>, const MTU: usize>(
    transmitter: &mut T,
    payload: &[u8],
    kind: SessionKind,
    priority: TransferPriority,
) -> Result<(), T::Error> {
    let can_id = can_id_for_session_kind(kind, priority);
    let breakdown = Breakdown::new(payload, can_id);

    transmitter.ensure_available_space(breakdown.frames_count())?;
    for frame in breakdown {
        transmitter.transmit(frame)?;
    }

    Ok(())
}
