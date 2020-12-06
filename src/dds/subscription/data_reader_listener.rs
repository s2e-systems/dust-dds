use crate::infrastructure::qos::DataReaderQos;

use crate::infrastructure::status::{SampleRejectedStatus, LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SubscriptionMatchedStatus, SampleLostStatus};
use crate::subscription::data_reader::DataReader;
use crate::types::DDSType;

pub trait DataReaderListener<T: DDSType>{
    fn on_data_available(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>);
    fn on_sample_rejected(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>, status: SampleRejectedStatus);
    fn on_liveliness_changed(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>, status: LivelinessChangedStatus);
    fn on_requested_deadline_missed(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>, status: RequestedDeadlineMissedStatus);
    fn on_requested_incompatible_qos(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>, status: RequestedIncompatibleQosStatus);
    fn on_subscription_matched(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>, status: SubscriptionMatchedStatus);
    fn on_sample_lost(&self, the_reader: dyn DataReader<T, Qos=DataReaderQos, Listener=dyn DataReaderListener<T>>, status: SampleLostStatus);
}