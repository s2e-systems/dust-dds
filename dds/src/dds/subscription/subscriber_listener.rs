use crate::infrastructure::status::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::{data_reader::AnyDataReader, subscriber::Subscriber};

/// This trait represents a listener object which can be associated with the ['Subscriber'](super::publisher::Publisher) entity.
pub trait SubscriberListener {
    /// Method that is called when any reader belonging to this subcriber reports new data available. This method is triggered before the on_data_available method.
    fn on_data_on_readers(&mut self, _the_subscriber: &Subscriber) {}

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
