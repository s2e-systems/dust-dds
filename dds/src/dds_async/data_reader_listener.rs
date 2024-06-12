use std::{future::Future, pin::Pin};

use crate::infrastructure::status::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::data_reader::DataReaderAsync;

/// This trait represents a listener object which can be associated with the [`DataReaderAsync`] entity.
pub trait DataReaderListenerAsync<'a>
where
    Self: 'static,
{
    /// Type of the DataReader with which this Listener will be associated.
    type Foo;

    /// Method that is called when new data is received by the reader.
    fn on_data_available(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when this reader reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
        _status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
    /// Method that is called when this reader reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
        _status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when this reader reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
        _status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when this reader reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
        _status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when this reader reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
        _status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }

    /// Method that is called when this reader reports a sample lost status.
    fn on_sample_lost(
        &mut self,
        _the_reader: DataReaderAsync<Self::Foo>,
        _status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(std::future::ready(()))
    }
}
