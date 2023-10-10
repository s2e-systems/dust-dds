use crate::implementation::{
    actors::{domain_participant_actor::DomainParticipantActor, topic_actor::TopicActor},
    utils::actor::ActorAddress,
};

#[derive(Clone, PartialEq, Eq)]
pub struct DdsTopic {
    topic_address: ActorAddress<TopicActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl DdsTopic {
    pub fn new(
        topic_address: ActorAddress<TopicActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            topic_address,
            participant_address,
        }
    }

    pub fn topic_address(&self) -> &ActorAddress<TopicActor> {
        &self.topic_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}
