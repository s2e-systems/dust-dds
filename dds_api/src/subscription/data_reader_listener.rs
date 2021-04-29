use crate::{
    dcps_psm::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    infrastructure::listener::Listener,
};

use super::data_reader::DataReader;

pub trait DataReaderListener: Listener {
    type DataType;
    fn on_data_available(&self, the_reader: &dyn DataReader<Self::DataType>);
    fn on_sample_rejected(
        &self,
        the_reader: &dyn DataReader<Self::DataType>,
        status: SampleRejectedStatus,
    );
    fn on_liveliness_changed(
        &self,
        the_reader: &dyn DataReader<Self::DataType>,
        status: LivelinessChangedStatus,
    );
    fn on_requested_deadline_missed(
        &self,
        the_reader: &dyn DataReader<Self::DataType>,
        status: RequestedDeadlineMissedStatus,
    );
    fn on_requested_incompatible_qos(
        &self,
        the_reader: &dyn DataReader<Self::DataType>,
        status: RequestedIncompatibleQosStatus,
    );
    fn on_subscription_matched(
        &self,
        the_reader: &dyn DataReader<Self::DataType>,
        status: SubscriptionMatchedStatus,
    );
    fn on_sample_lost(&self, the_reader: &dyn DataReader<Self::DataType>, status: SampleLostStatus);
}
