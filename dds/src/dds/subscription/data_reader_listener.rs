use crate::infrastructure::status::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

use super::data_reader::DataReader;

pub trait DataReaderListener<Foo> {
    fn on_data_available(&mut self, _the_reader: &DataReader<Foo>) {}
    fn on_sample_rejected(&mut self, _the_reader: &DataReader<Foo>, _status: SampleRejectedStatus) {
    }
    fn on_liveliness_changed(
        &mut self,
        _the_reader: &DataReader<Foo>,
        _status: LivelinessChangedStatus,
    ) {
    }
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &DataReader<Foo>,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &DataReader<Foo>,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }
    fn on_subscription_matched(
        &mut self,
        _the_reader: &DataReader<Foo>,
        _status: SubscriptionMatchedStatus,
    ) {
    }
    fn on_sample_lost(&mut self, _the_reader: &DataReader<Foo>, _status: SampleLostStatus) {}
}
