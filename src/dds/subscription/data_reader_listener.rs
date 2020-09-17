use std::any::Any;
use crate::dds::infrastructure::status::{SampleRejectedStatus, LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SubscriptionMatchedStatus, SampleLostStatus};
use crate::dds::subscription::data_reader::DataReader;

pub trait DataReaderListener<T>: Any + Send + Sync{
    fn on_data_available(&self, the_reader: DataReader<T>);
    fn on_sample_rejected(&self, the_reader: DataReader<T>, status: SampleRejectedStatus);
    fn on_liveliness_changed(&self, the_reader: DataReader<T>, status: LivelinessChangedStatus);
    fn on_requested_deadline_missed(&self, the_reader: DataReader<T>, status: RequestedDeadlineMissedStatus);
    fn on_requested_incompatible_qos(&self, the_reader: DataReader<T>, status: RequestedIncompatibleQosStatus);
    fn on_subscription_matched(&self, the_reader: DataReader<T>, status: SubscriptionMatchedStatus);
    fn on_sample_lost(&self, the_reader: DataReader<T>, status: SampleLostStatus);
}

