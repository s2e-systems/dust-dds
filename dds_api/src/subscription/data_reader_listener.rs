use crate::dcps_psm::{
    LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
    SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
};

pub trait DataReaderListener {
    fn on_data_available(&self) {}
    fn on_sample_rejected(&self, _status: SampleRejectedStatus) {}
    fn on_liveliness_changed(&self, _status: LivelinessChangedStatus) {}
    fn on_requested_deadline_missed(&self, _status: RequestedDeadlineMissedStatus) {}
    fn on_requested_incompatible_qos(&self, _status: RequestedIncompatibleQosStatus) {}
    fn on_subscription_matched(&self, _status: SubscriptionMatchedStatus) {}
    fn on_sample_lost(&self, _status: SampleLostStatus) {}
}
