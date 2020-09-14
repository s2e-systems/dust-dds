use std::any::Any;
use crate::dds::infrastructure::status::{SampleRejectedStatus, LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SubscriptionMatchedStatus, SampleLostStatus};
use crate::dds::subscription::data_reader::DataReader;

pub trait DataReaderListener: Any + Send + Sync{
    fn on_data_available(&self, the_reader: DataReader);
    fn on_sample_rejected(&self, the_reader: DataReader, status: SampleRejectedStatus);
    fn on_liveliness_changed(&self, the_reader: DataReader, status: LivelinessChangedStatus);
    fn on_requested_deadline_missed(&self, the_reader: DataReader, status: RequestedDeadlineMissedStatus);
    fn on_requested_incompatible_qos(&self, the_reader: DataReader, status: RequestedIncompatibleQosStatus);
    fn on_subscription_matched(&self, the_reader: DataReader, status: SubscriptionMatchedStatus);
    fn on_sample_lost(&self, the_reader: DataReader, status: SampleLostStatus);
}

