use crate::implementation::{
    actors::domain_participant_actor::DomainParticipantActor, utils::actor::ActorAddress,
};

#[derive(Clone, PartialEq, Eq)]
pub struct DomainParticipantNode {
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl DomainParticipantNode {
    pub fn new(participant_address: ActorAddress<DomainParticipantActor>) -> Self {
        Self {
            participant_address,
        }
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}
