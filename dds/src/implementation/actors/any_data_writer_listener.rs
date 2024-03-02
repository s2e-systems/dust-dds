use crate::{
    dds_async::{data_writer::DataWriterAsync, publisher::PublisherAsync, topic::TopicAsync},
    implementation::utils::actor::ActorAddress,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
    publication::{data_writer::DataWriter, data_writer_listener::DataWriterListener},
};

use super::{data_writer_actor::DataWriterActor, status_condition_actor::StatusConditionActor};

pub trait AnyDataWriterListener {
    #[allow(dead_code)]
    fn trigger_on_liveliness_lost(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: LivelinessLostStatus,
    );
    #[allow(dead_code)]
    fn trigger_on_offered_deadline_missed(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedDeadlineMissedStatus,
    );
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedIncompatibleQosStatus,
    );
    fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: PublicationMatchedStatus,
    );
}

impl<T> AnyDataWriterListener for T
where
    T: DataWriterListener,
{
    fn trigger_on_liveliness_lost(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: LivelinessLostStatus,
    ) {
        self.on_liveliness_lost(
            &DataWriter::new(DataWriterAsync::new(
                writer_address,
                status_condition_address,
                publisher,
                topic,
            )),
            status,
        );
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedDeadlineMissedStatus,
    ) {
        self.on_offered_deadline_missed(
            &DataWriter::new(DataWriterAsync::new(
                writer_address,
                status_condition_address,
                publisher,
                topic,
            )),
            status,
        );
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedIncompatibleQosStatus,
    ) {
        self.on_offered_incompatible_qos(
            &DataWriter::new(DataWriterAsync::new(
                writer_address,
                status_condition_address,
                publisher,
                topic,
            )),
            status,
        );
    }

    fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: PublicationMatchedStatus,
    ) {
        self.on_publication_matched(
            &DataWriter::new(DataWriterAsync::new(
                writer_address,
                status_condition_address,
                publisher,
                topic,
            )),
            status,
        )
    }
}
