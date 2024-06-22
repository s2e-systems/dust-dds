use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{data_reader::DataReaderAsync, data_reader_listener::DataReaderListenerAsync},
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::data_reader::DataReader;

/// This trait represents a listener object which can be associated with the [`DataReader`] entity.
pub trait DataReaderListener<'a>: 'static {
    /// Type of the DataReader with which this Listener will be associated.
    type Foo;

    /// Method that is called when new data is received by the reader.
    fn on_data_available(&mut self, _the_reader: DataReader<Self::Foo>) {}

    /// Method that is called when this reader reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: SampleRejectedStatus,
    ) {
    }
    /// Method that is called when this reader reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: LivelinessChangedStatus,
    ) {
    }

    /// Method that is called when this reader reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when this reader reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when this reader reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReader<Self::Foo>,
        _status: SubscriptionMatchedStatus,
    ) {
    }

    /// Method that is called when this reader reports a sample lost status.
    fn on_sample_lost(&mut self, _the_reader: DataReader<Self::Foo>, _status: SampleLostStatus) {}
}

impl<'a, Foo> DataReaderListenerAsync<'_> for Box<dyn DataReaderListener<'_, Foo = Foo> + Send + 'a>
where
    Self: 'static,
{
    type Foo = Foo;

    fn on_data_available(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_data_available(self.as_mut(), DataReader::new(the_reader));
        Box::pin(std::future::ready(()))
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_sample_rejected(self.as_mut(), DataReader::new(the_reader), status);
        Box::pin(std::future::ready(()))
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_liveliness_changed(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_requested_deadline_missed(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_requested_incompatible_qos(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_subscription_matched(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_sample_lost(
        &mut self,
        the_reader: DataReaderAsync<Self::Foo>,
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DataReaderListener::on_sample_lost(self.as_mut(), DataReader::new(the_reader), status);
        Box::pin(std::future::ready(()))
    }
}
