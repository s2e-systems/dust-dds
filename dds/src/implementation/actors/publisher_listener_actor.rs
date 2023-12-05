use dust_dds_derive::actor_interface;

use crate::{
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{OfferedIncompatibleQosStatus, PublicationMatchedStatus},
    publication::{data_writer::DataWriter, publisher_listener::PublisherListener},
};

use super::{
    data_writer_actor::DataWriterActor, domain_participant_actor::DomainParticipantActor,
    publisher_actor::PublisherActor,
};

pub struct PublisherListenerActor {
    listener: Box<dyn PublisherListener + Send>,
}

impl PublisherListenerActor {
    pub fn new(listener: Box<dyn PublisherListener + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl PublisherListenerActor {
    async fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.listener.on_offered_incompatible_qos(
            &DataWriter::<()>::new(writer_address, publisher_address, participant_address),
            status,
        )
    }

    async fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: PublicationMatchedStatus,
    ) {
        self.listener.on_publication_matched(
            &DataWriter::<()>::new(writer_address, publisher_address, participant_address),
            status,
        )
    }
}
