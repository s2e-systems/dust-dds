use crate::dcps_psm::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::{data_reader::AnyDataReader, subscriber::Subscriber};

pub trait SubscriberListener {
    fn on_data_on_readers(&mut self, _the_subscriber: &Subscriber) {}
    fn on_data_available(&mut self, _the_reader: &dyn AnyDataReader) {}
    fn on_sample_rejected(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SampleRejectedStatus,
    ) {
    }
    fn on_liveliness_changed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: LivelinessChangedStatus,
    ) {
    }
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }
    fn on_subscription_matched(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SubscriptionMatchedStatus,
    ) {
    }
    fn on_sample_lost(&mut self, _the_reader: &dyn AnyDataReader, _status: SampleLostStatus) {}
}
