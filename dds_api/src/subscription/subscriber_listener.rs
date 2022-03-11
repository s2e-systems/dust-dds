use crate::dcps_psm::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::data_reader::AnyDataReader;

pub trait SubscriberListener {
    fn on_data_on_readers(&self) {}
    fn on_data_available(&self, _the_reader: &dyn AnyDataReader) {}
    fn on_sample_rejected(&self, _the_reader: &dyn AnyDataReader, _status: SampleRejectedStatus) {}
    fn on_liveliness_changed(
        &self,
        _the_reader: &dyn AnyDataReader,
        _status: LivelinessChangedStatus,
    ) {
    }
    fn on_requested_deadline_missed(
        &self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }
    fn on_requested_incompatible_qos(
        &self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }
    fn on_subscription_matched(
        &self,
        _the_reader: &dyn AnyDataReader,
        _status: SubscriptionMatchedStatus,
    ) {
    }
    fn on_sample_lost(&self, _the_reader: &dyn AnyDataReader, _status: SampleLostStatus) {}
}
