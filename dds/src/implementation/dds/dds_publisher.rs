use crate::implementation::{
    actors::{domain_participant_actor::DomainParticipantActor, publisher_actor::PublisherActor},
    utils::actor::ActorAddress,
};

#[derive(Clone, PartialEq, Eq)]
pub struct DdsPublisher {
    publisher_address: ActorAddress<PublisherActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl DdsPublisher {
    pub fn new(
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            publisher_address,
            participant_address,
        }
    }

    pub fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        &self.publisher_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}
