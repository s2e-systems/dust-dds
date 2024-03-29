use std::{future::Future, pin::Pin};

use crate::{
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    subscription::data_reader::AnyDataReader,
};

use super::subscriber::SubscriberAsync;

/// This trait represents a listener object which can be associated with the [`SubscriberAsync`](super::subscriber::Subscriber) entity.
pub trait SubscriberListenerAsync {
    /// Method that is called when any reader belonging to this subcriber reports new data available. This method is triggered before the on_data_available method.
    fn on_data_on_readers(
        &mut self,
        _the_subscriber: SubscriberAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports new data available.
    fn on_data_available<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample rejected status.
    fn on_sample_rejected<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a liveliness changed status.
    fn on_liveliness_changed<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested deadline missed status.
    fn on_requested_deadline_missed<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a subscription matched status.
    fn on_subscription_matched<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample lost status.
    fn on_sample_lost<'a, 'b>(
        &'a mut self,
        _the_reader: &'b (dyn AnyDataReader + Sync),
        _status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
    {
        Box::pin(std::future::ready(()))
    }
}
