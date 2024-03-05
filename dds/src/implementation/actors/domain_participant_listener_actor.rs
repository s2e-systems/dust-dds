use std::{future::Future, pin::Pin};

use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{domain_participant_listener::DomainParticipantListenerAsync, topic::TopicAsync},
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
};

pub trait DomainParticipantListenerAsyncDyn {
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_sample_lost<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_data_available<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_sample_rejected<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_liveliness_changed<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_requested_deadline_missed<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_requested_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;

    fn on_subscription_matched<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b;
}

impl<T> DomainParticipantListenerAsyncDyn for T
where
    T: DomainParticipantListenerAsync + Send,
{
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async { self.on_inconsistent_topic(the_topic, status).await })
    }

    fn on_liveliness_lost<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_liveliness_lost(the_writer, status).await })
    }

    fn on_offered_deadline_missed<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_offered_deadline_missed(the_writer, status).await })
    }

    fn on_offered_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_offered_incompatible_qos(the_writer, status).await })
    }

    fn on_sample_lost<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_sample_lost(the_reader, status).await })
    }

    fn on_data_available<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_data_available(the_reader).await })
    }

    fn on_sample_rejected<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_sample_rejected(the_reader, status).await })
    }

    fn on_liveliness_changed<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_liveliness_changed(the_reader, status).await })
    }

    fn on_requested_deadline_missed<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_requested_deadline_missed(the_reader, status).await })
    }

    fn on_requested_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_requested_incompatible_qos(the_reader, status).await })
    }

    fn on_publication_matched<'a, 'b>(
        &'a mut self,
        the_writer: &'b (dyn AnyDataWriter + Sync),
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_publication_matched(the_writer, status).await })
    }

    fn on_subscription_matched<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(async { self.on_subscription_matched(the_reader, status).await })
    }
}

pub struct DomainParticipantListenerActor {
    listener: Box<dyn DomainParticipantListenerAsyncDyn + Send>,
}

impl DomainParticipantListenerActor {
    pub fn new(listener: Box<dyn DomainParticipantListenerAsyncDyn + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl DomainParticipantListenerActor {
    async fn trigger_on_sample_rejected(&mut self, status: SampleRejectedStatus) {
        self.listener.on_sample_rejected(&(), status).await
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        status: RequestedIncompatibleQosStatus,
    ) {
        self.listener
            .on_requested_incompatible_qos(&(), status)
            .await
    }

    async fn trigger_on_offered_incompatible_qos(&mut self, status: OfferedIncompatibleQosStatus) {
        self.listener.on_offered_incompatible_qos(&(), status).await
    }

    async fn trigger_on_publication_matched(&mut self, status: PublicationMatchedStatus) {
        self.listener.on_publication_matched(&(), status).await
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        status: RequestedDeadlineMissedStatus,
    ) {
        self.listener
            .on_requested_deadline_missed(&(), status)
            .await
    }

    async fn trigger_on_subscription_matched(&mut self, status: SubscriptionMatchedStatus) {
        self.listener.on_subscription_matched(&(), status).await
    }

    async fn trigger_on_sample_lost(&mut self, status: SampleLostStatus) {
        self.listener.on_sample_lost(&(), status).await
    }
}
