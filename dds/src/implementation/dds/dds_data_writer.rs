use crate::{
    implementation::{
        actors::{
            data_writer_actor::DataWriterActor, domain_participant_actor::DomainParticipantActor,
            publisher_actor::PublisherActor,
        },
        utils::actor::ActorAddress,
    },
    publication::data_writer::AnyDataWriter,
};

#[derive(Clone, PartialEq, Eq)]
pub struct DataWriterNode {
    writer_address: ActorAddress<DataWriterActor>,
    publisher_address: ActorAddress<PublisherActor>,
    participant_address: ActorAddress<DomainParticipantActor>,
}

impl DataWriterNode {
    pub fn new(
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
    ) -> Self {
        Self {
            writer_address,
            publisher_address,
            participant_address,
        }
    }

    pub fn writer_address(&self) -> &ActorAddress<DataWriterActor> {
        &self.writer_address
    }

    pub fn publisher_address(&self) -> &ActorAddress<PublisherActor> {
        &self.publisher_address
    }

    pub fn participant_address(&self) -> &ActorAddress<DomainParticipantActor> {
        &self.participant_address
    }
}

impl AnyDataWriter for DataWriterNode {}
