pub mod error;
pub mod message;
pub mod node_id;
pub mod service;
pub mod service_id;
pub mod session_id;
pub mod session_kind;
pub mod subject_id;
pub mod transfer_priority;

// TODO: Consider adding a Bounded trait + derive macro for the general case
// that is used for simple structures such as node_id and service_id.
// That trait should define their defined ranges and provide a TryFrom
// implementation as this is currently done by hand.
// A lighter change that may be a better fit is to at least provide constants
// for the lower and higher bound of those structures, as this bound is
// currently implicit and written as a literal in various places, such as tests.

pub use message::MessageSessionId;
pub use session_id::SessionId;
// TODO: can_id_for_session_kind should be removed and substituted with a SessionKind.as_u32 or a From<SessionKind> for u32.
pub use node_id::NodeId;
pub use session_kind::{can_id_for_session_kind, SessionKind};
pub use subject_id::SubjectId;
pub use transfer_priority::TransferPriority;
