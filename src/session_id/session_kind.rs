use super::{
    node_id::NodeId, service::ServiceSessionId, service_id::ServiceId, subject_id::SubjectId,
    MessageSessionId, SessionId, TransferPriority,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Request {
    source_node_id: NodeId,
    destination_node_id: NodeId,
    service_id: ServiceId,
}

impl Request {
    pub fn new(source_node_id: NodeId, destination_node_id: NodeId, service_id: ServiceId) -> Self {
        Self {
            source_node_id,
            destination_node_id,
            service_id,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SessionKind {
    Message {
        source_node_id: NodeId,
        subject_id: SubjectId,
    },
    Request(Request),
    Response(Request),
}

impl From<SessionId> for SessionKind {
    fn from(id: SessionId) -> Self {
        match id {
            SessionId::Message(message) => SessionKind::Message {
                source_node_id: message.source_node_id(),
                subject_id: message.subject_id(),
            },
            SessionId::Rpc(service) => {
                let request = Request {
                    source_node_id: service.source_node_id(),
                    destination_node_id: service.destination_node_id(),
                    service_id: service.service_id(),
                };

                if service.is_request() {
                    SessionKind::Request(request)
                } else {
                    SessionKind::Response(Request{
                        source_node_id: request.destination_node_id,
                        destination_node_id: request.source_node_id,
                        ..request
                    })
                }
            }
        }
    }
}

// TODO: It might be better to have a SessionKind contain the priority itself.
// If that is done, then this function should be converted to a
// `From<SessionKind>` implementation for `u32`.
pub fn can_id_for_session_kind(kind: SessionKind, priority: TransferPriority) -> u32 {
    match kind {
        SessionKind::Message {
            source_node_id,
            subject_id,
        } => MessageSessionId::as_u32(source_node_id, subject_id, priority),
        SessionKind::Request(Request {
            source_node_id,
            destination_node_id,
            service_id,
        }) => ServiceSessionId::request_as_u32(
            source_node_id,
            destination_node_id,
            service_id,
            priority,
        ),
        SessionKind::Response(Request {
            source_node_id,
            destination_node_id,
            service_id,
        }) => ServiceSessionId::response_as_u32(
            destination_node_id,
            source_node_id,
            service_id,
            priority,
        ),
    }
}

#[cfg(test)]
pub mod strategy {
    use super::super::node_id::strategy::node_id;
    use super::super::service_id::strategy::service_id;
    use super::super::subject_id::strategy::subject_id;
    use super::*;
    use proptest::{prop_compose, prop_oneof, strategy::Strategy};

    pub fn session_kind() -> impl Strategy<Value = SessionKind> {
        prop_oneof![
            (node_id(), subject_id()).prop_map(|(source_node_id, subject_id)| {
                SessionKind::Message {
                    source_node_id,
                    subject_id,
                }
            }),
            request().prop_map(|request| SessionKind::Request(request)),
            request().prop_map(|request| SessionKind::Response(request)),
        ]
    }

    prop_compose! {
        pub fn request()(source in node_id(), destination in node_id(), service in service_id()) -> Request {
            Request::new(source, destination, service)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::strategy::session_kind;
    use super::super::transfer_priority::strategy::transfer_priority;
    use proptest::prelude::*;

    extern crate std;
    use std::format;

    proptest! {
        #[test]
        fn converting_a_session_kind_to_a_number_and_then_back_preserves_it(
            original_session_kind in session_kind(),
            priority in transfer_priority()
        ) {
            let session_id = SessionId::from(can_id_for_session_kind(original_session_kind, priority));
            let restored_session_kind = SessionKind::from(session_id);

            prop_assert!(restored_session_kind == original_session_kind);
        }
    }
}
