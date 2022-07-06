use crate::dcps_psm::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::data_reader::FooDataReader;

pub trait DataReaderListener {
    type Foo;

    fn on_data_available(&mut self, _the_reader: &dyn FooDataReader<Self::Foo>) {}
    fn on_sample_rejected(
        &mut self,
        _the_reader: &dyn FooDataReader<Self::Foo>,
        _status: SampleRejectedStatus,
    ) {
    }
    fn on_liveliness_changed(
        &mut self,
        _the_reader: &dyn FooDataReader<Self::Foo>,
        _status: LivelinessChangedStatus,
    ) {
    }
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &dyn FooDataReader<Self::Foo>,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &dyn FooDataReader<Self::Foo>,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }
    fn on_subscription_matched(
        &mut self,
        _the_reader: &dyn FooDataReader<Self::Foo>,
        _status: SubscriptionMatchedStatus,
    ) {
    }
    fn on_sample_lost(
        &mut self,
        _the_reader: &dyn FooDataReader<Self::Foo>,
        _status: SampleLostStatus,
    ) {
    }
}
