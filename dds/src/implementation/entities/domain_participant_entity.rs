use crate::implementation::{
    actors::domain_participant_actor::DomainParticipantActor, utils::actor::ActorAddress,
};

#[derive(Clone)]
pub struct DomainParticipantEntity {
    participant_address: ActorAddress<DomainParticipantActor>,
    runtime_handle: tokio::runtime::Handle,
}

impl DomainParticipantEntity {
    pub fn new(
        participant_address: ActorAddress<DomainParticipantActor>,
        runtime_handle: tokio::runtime::Handle,
    ) -> Self {
        Self {
            participant_address,
            runtime_handle,
        }
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }

    pub fn runtime_handle(&self) -> &tokio::runtime::Handle {
        &self.runtime_handle
    }
}
