use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_writer::DataWriterAsync, data_writer_listener::DataWriterListenerAsync,
        publisher::PublisherAsync, topic::TopicAsync,
    },
    implementation::actor::ActorAddress,
    infrastructure::status::{
        LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
        PublicationMatchedStatus,
    },
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
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    #[allow(dead_code)]
    fn trigger_on_offered_deadline_missed(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
    fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;
}

impl<'a, Foo> AnyDataWriterListener for Box<dyn DataWriterListenerAsync<Foo = Foo> + Send + 'a>
where
    Foo: 'a,
{
    fn trigger_on_liveliness_lost(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            self.on_liveliness_lost(
                DataWriterAsync::new(writer_address, status_condition_address, publisher, topic),
                status,
            )
            .await
        })
    }

    fn trigger_on_offered_deadline_missed(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            self.on_offered_deadline_missed(
                DataWriterAsync::new(writer_address, status_condition_address, publisher, topic),
                status,
            )
            .await
        })
    }

    fn trigger_on_offered_incompatible_qos(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            self.on_offered_incompatible_qos(
                DataWriterAsync::new(writer_address, status_condition_address, publisher, topic),
                status,
            )
            .await
        })
    }

    fn trigger_on_publication_matched(
        &mut self,
        writer_address: ActorAddress<DataWriterActor>,
        status_condition_address: ActorAddress<StatusConditionActor>,
        publisher: PublisherAsync,
        topic: TopicAsync,
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async {
            self.on_publication_matched(
                DataWriterAsync::new(writer_address, status_condition_address, publisher, topic),
                status,
            )
            .await
        })
    }
}
