use std::{future::Future, pin::Pin};

use crate::{
    dcps::runtime::DdsRuntime,
    dds_async::{data_reader::DataReaderAsync, subscriber::SubscriberAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

/// This trait represents a listener object which can be associated with the [`Subscriber`](super::subscriber::Subscriber) entity.
pub trait SubscriberListener<R: DdsRuntime> {
    /// Method that is called when any reader belonging to this subcriber reports new data available. This method is triggered before the on_data_available method.
    fn on_data_on_readers(
        &mut self,
        _the_subscriber: SubscriberAsync<R>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports new data available.
    fn on_data_available(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        _status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        _status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        _status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        _status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        _status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample lost status.
    fn on_sample_lost(
        &mut self,
        _the_reader: DataReaderAsync<R, ()>,
        _status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
}
