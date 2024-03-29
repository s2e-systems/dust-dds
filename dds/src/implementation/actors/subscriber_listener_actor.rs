use std::{future::Future, pin::Pin};

use dust_dds_derive::actor_interface;

use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::data_reader::AnyDataReader,
};

pub trait SubscriberListenerAsyncDyn {
    fn on_data_on_readers(
        &mut self,
        the_subscriber: SubscriberAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

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

    fn on_subscription_matched<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SubscriptionMatchedStatus,
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
}

impl<T> SubscriberListenerAsyncDyn for T
where
    T: SubscriberListenerAsync + Send,
{
    fn on_data_on_readers(
        &mut self,
        the_subscriber: SubscriberAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async { self.on_data_on_readers(the_subscriber).await })
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
}

pub struct SubscriberListenerActor {
    listener: Box<dyn SubscriberListenerAsyncDyn + Send>,
}

impl SubscriberListenerActor {
    pub fn new(listener: Box<dyn SubscriberListenerAsyncDyn + Send>) -> Self {
        Self { listener }
    }
}

#[actor_interface]
impl SubscriberListenerActor {
    async fn trigger_on_data_on_readers(&mut self, subscriber: SubscriberAsync) -> () {
        self.listener.on_data_on_readers(subscriber).await
    }

    async fn trigger_on_sample_rejected(&mut self, status: SampleRejectedStatus) -> () {
        self.listener.on_sample_rejected(&(), status).await
    }

    async fn trigger_on_requested_incompatible_qos(
        &mut self,
        status: RequestedIncompatibleQosStatus,
    ) -> () {
        self.listener
            .on_requested_incompatible_qos(&(), status)
            .await
    }

    async fn trigger_on_requested_deadline_missed(
        &mut self,
        status: RequestedDeadlineMissedStatus,
    ) -> () {
        self.listener
            .on_requested_deadline_missed(&(), status)
            .await
    }

    async fn trigger_on_subscription_matched(&mut self, status: SubscriptionMatchedStatus) -> () {
        self.listener.on_subscription_matched(&(), status).await
    }

    async fn trigger_on_sample_lost(&mut self, status: SampleLostStatus) -> () {
        self.listener.on_sample_lost(&(), status).await
    }
}
