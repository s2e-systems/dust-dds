use crate::implementation::{
    actors::{domain_participant_actor::DomainParticipantActor, subscriber_actor::SubscriberActor},
    utils::actor::ActorAddress,
};

#[derive(Clone, PartialEq, Eq)]
pub struct SubscriberNode {
    subscriber_address: ActorAddress<SubscriberActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl SubscriberNode {
    pub fn new(
        subscriber_address: ActorAddress<SubscriberActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            subscriber_address,
            participant_address,
        }
    }

    pub fn subscriber_address(&self) -> &ActorAddress<SubscriberActor> {
        &self.subscriber_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}
