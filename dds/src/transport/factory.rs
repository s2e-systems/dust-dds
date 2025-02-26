use super::{participant::TransportParticipant, types::GuidPrefix};

pub trait TransportParticipantFactory: Send + Sync {
    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Box<dyn TransportParticipant>;
}
