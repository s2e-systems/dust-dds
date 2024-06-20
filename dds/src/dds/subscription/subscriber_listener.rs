use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::{data_reader::AnyDataReader, subscriber::Subscriber};

/// This trait represents a listener object which can be associated with the [`Subscriber`](super::subscriber::Subscriber) entity.
pub trait SubscriberListener {
    /// Method that is called when any reader belonging to this subcriber reports new data available. This method is triggered before the on_data_available method.
    fn on_data_on_readers(&mut self, _the_subscriber: Subscriber) {}

    /// Method that is called when any reader belonging to this subcriber reports new data available.
    fn on_data_available(&mut self, _the_reader: &dyn AnyDataReader) {}

    /// Method that is called when any reader belonging to this subcriber reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SampleRejectedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: LivelinessChangedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SubscriptionMatchedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample lost status.
    fn on_sample_lost(&mut self, _the_reader: &dyn AnyDataReader, _status: SampleLostStatus) {}
}

impl SubscriberListenerAsync for Box<dyn SubscriberListener + Send> {
    fn on_data_on_readers(
        &mut self,
        the_subscriber: SubscriberAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_data_on_readers(self.as_mut(), Subscriber::new(the_subscriber));
        Box::pin(std::future::ready(()))
    }

    fn on_data_available<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_data_available(self.as_mut(), the_reader);
        Box::pin(std::future::ready(()))
    }

    fn on_sample_rejected<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_sample_rejected(self.as_mut(), the_reader, status);
        Box::pin(std::future::ready(()))
    }

    fn on_liveliness_changed<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_liveliness_changed(self.as_mut(), the_reader, status);
        Box::pin(std::future::ready(()))
    }

    fn on_requested_deadline_missed<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_requested_deadline_missed(self.as_mut(), the_reader, status);
        Box::pin(std::future::ready(()))
    }

    fn on_requested_incompatible_qos<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_requested_incompatible_qos(self.as_mut(), the_reader, status);
        Box::pin(std::future::ready(()))
    }

    fn on_subscription_matched<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_subscription_matched(self.as_mut(), the_reader, status);
        Box::pin(std::future::ready(()))
    }

    fn on_sample_lost<'a, 'b>(
        &'a mut self,
        the_reader: &'b (dyn AnyDataReader + Sync),
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        SubscriberListener::on_sample_lost(self.as_mut(), the_reader, status);
        Box::pin(std::future::ready(()))
    }
}
