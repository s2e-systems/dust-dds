use super::types::GuidPrefix;

pub trait TransportParticipantFactory: Send {
    type TransportParticipant;

    fn create_participant(
        &self,
        guid_prefix: GuidPrefix,
        domain_id: i32,
    ) -> Self::TransportParticipant;
}
