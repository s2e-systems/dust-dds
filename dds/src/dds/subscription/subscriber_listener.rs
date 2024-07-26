use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, subscriber::SubscriberAsync,
        subscriber_listener::SubscriberListenerAsync,
    },
    infrastructure::status::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
};

use super::{data_reader::DataReader, subscriber::Subscriber};

/// This trait represents a listener object which can be associated with the [`Subscriber`](super::subscriber::Subscriber) entity.
pub trait SubscriberListener {
    /// Method that is called when any reader belonging to this subcriber reports new data available. This method is triggered before the on_data_available method.
    fn on_data_on_readers(&mut self, _the_subscriber: Subscriber) {}

    /// Method that is called when any reader belonging to this subcriber reports new data available.
    fn on_data_available(&mut self, _the_reader: DataReader<()>) {}

    /// Method that is called when any reader belonging to this subcriber reports a sample rejected status.
    fn on_sample_rejected(&mut self, _the_reader: DataReader<()>, _status: SampleRejectedStatus) {}

    /// Method that is called when any reader belonging to this subcriber reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReader<()>,
        _status: LivelinessChangedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReader<()>,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReader<()>,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReader<()>,
        _status: SubscriptionMatchedStatus,
    ) {
    }

    /// Method that is called when any reader belonging to this subcriber reports a sample lost status.
    fn on_sample_lost(&mut self, _the_reader: DataReader<()>, _status: SampleLostStatus) {}
}

impl SubscriberListenerAsync for Box<dyn SubscriberListener + Send> {
    fn on_data_on_readers(
        &mut self,
        the_subscriber: SubscriberAsync,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_data_on_readers(self.as_mut(), Subscriber::new(the_subscriber));
        Box::pin(std::future::ready(()))
    }

    fn on_data_available(
        &mut self,
        the_reader: DataReaderAsync<()>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_data_available(self.as_mut(), DataReader::new(the_reader));
        Box::pin(std::future::ready(()))
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_sample_rejected(self.as_mut(), DataReader::new(the_reader), status);
        Box::pin(std::future::ready(()))
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_liveliness_changed(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_requested_deadline_missed(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_requested_incompatible_qos(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_subscription_matched(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_sample_lost(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        SubscriberListener::on_sample_lost(self.as_mut(), DataReader::new(the_reader), status);
        Box::pin(std::future::ready(()))
    }
}
