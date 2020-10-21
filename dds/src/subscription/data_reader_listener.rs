use crate::infrastructure::status::{SampleRejectedStatus, LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SubscriptionMatchedStatus, SampleLostStatus};
use crate::subscription::data_reader::DataReader;
use crate::infrastructure::listener::NoListener;

pub trait DataReaderListener<T>{
    fn on_data_available(&self, the_reader: DataReader<T>);
    fn on_sample_rejected(&self, the_reader: DataReader<T>, status: SampleRejectedStatus);
    fn on_liveliness_changed(&self, the_reader: DataReader<T>, status: LivelinessChangedStatus);
    fn on_requested_deadline_missed(&self, the_reader: DataReader<T>, status: RequestedDeadlineMissedStatus);
    fn on_requested_incompatible_qos(&self, the_reader: DataReader<T>, status: RequestedIncompatibleQosStatus);
    fn on_subscription_matched(&self, the_reader: DataReader<T>, status: SubscriptionMatchedStatus);
    fn on_sample_lost(&self, the_reader: DataReader<T>, status: SampleLostStatus);
}

impl<T> DataReaderListener<T> for NoListener {
    fn on_data_available(&self, _the_reader: DataReader<T>) {
        todo!()
    }

    fn on_sample_rejected(&self, _the_reader: DataReader<T>, _status: SampleRejectedStatus) {
        todo!()
    }

    fn on_liveliness_changed(&self, _the_reader: DataReader<T>, _status: LivelinessChangedStatus) {
        todo!()
    }

    fn on_requested_deadline_missed(&self, _the_reader: DataReader<T>, _status: RequestedDeadlineMissedStatus) {
        todo!()
    }

    fn on_requested_incompatible_qos(&self, _the_reader: DataReader<T>, _status: RequestedIncompatibleQosStatus) {
        todo!()
    }

    fn on_subscription_matched(&self, _the_reader: DataReader<T>, _status: SubscriptionMatchedStatus) {
        todo!()
    }

    fn on_sample_lost(&self, _the_reader: DataReader<T>, _status: SampleLostStatus) {
        todo!()
    }
}