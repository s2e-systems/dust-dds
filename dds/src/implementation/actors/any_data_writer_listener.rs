use crate::{
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
};

use super::{
    data_writer_actor::DataWriterActor, domain_participant_actor::DomainParticipantActor,
    publisher_actor::PublisherActor,
};

pub trait AnyDataWriterListener {
    fn trigger_on_liveliness_lost(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: LivelinessLostStatus,
    );
    fn trigger_on_offered_deadline_missed(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: PublicationMatchedStatus,
    );
}

impl<Foo> AnyDataWriterListener for Box<dyn DataWriterListener<Foo> + Send> {
    fn trigger_on_liveliness_lost(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(
            &DataWriter::new(writer_address, publisher_address, participant_address),
            status,
        );
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(
            &DataWriter::new(writer_address, publisher_address, participant_address),
            status,
        );
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(
            &DataWriter::new(writer_address, publisher_address, participant_address),
            status,
        );
    }

    fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        publisher_address: ActorAddress<PublisherActor>,
        participant_address: ActorAddress<DomainParticipantActor>,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(
            &DataWriter::new(writer_address, publisher_address, participant_address),
            status,
        )
    }
}
