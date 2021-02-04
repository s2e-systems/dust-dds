use crate::{
    dcps_psm::{
        LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
        SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    infrastructure::listener::Listener,
};

use super::{data_reader::AnyDataReader, subscriber::Subscriber};

pub trait SubscriberListener: Listener {
    fn on_data_on_readers(&self, the_subscriber: &dyn Subscriber);
    fn on_data_available(&self, the_reader: &dyn AnyDataReader);
    fn on_sample_rejected(&self, the_reader: &dyn AnyDataReader, status: SampleRejectedStatus);
    fn on_liveliness_changed(
        &self,
        the_reader: &dyn AnyDataReader,
        status: LivelinessChangedStatus,
    );
    fn on_requested_deadline_missed(
        &self,
        the_reader: &dyn AnyDataReader,
        status: RequestedDeadlineMissedStatus,
    );
    fn on_requested_incompatible_qos(
        &self,
        the_reader: &dyn AnyDataReader,
        status: RequestedIncompatibleQosStatus,
    );
    fn on_subscription_matched(
        &self,
        the_reader: &dyn AnyDataReader,
        status: SubscriptionMatchedStatus,
    );
    fn on_sample_lost(&self, the_reader: &dyn AnyDataReader, status: SampleLostStatus);
}
