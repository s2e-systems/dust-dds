use crate::{
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::data_writer::AnyDataWriter,
    subscription::{data_reader::AnyDataReader, subscriber::Subscriber},
    topic_definition::topic::AnyTopic,
};

/// The purpose of the DomainParticipantListener is to be the listener of last resort that is notified of all status changes not
/// captured by more specific listeners attached to the DomainEntity objects. When a relevant status change occurs, the DCPS
/// Service will first attempt to notify the listener attached to the concerned DomainEntity if one is installed. Otherwise, the
/// DCPS Service will notify the Listener attached to the DomainParticipant.
pub trait DomainParticipantListener {
    fn on_inconsistent_topic(
        &mut self,
        _the_topic: &dyn AnyTopic,
        _status: InconsistentTopicStatus,
    ) {
    }

    fn on_liveliness_lost(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: LivelinessLostStatus,
    ) {
    }

    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }

    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }

    fn on_data_on_readers(&mut self, _the_subscriber: &Subscriber) {}

    fn on_sample_lost(&mut self, _the_reader: &dyn AnyDataReader, _status: SampleLostStatus) {}

    fn on_data_available(&mut self, _the_reader: &dyn AnyDataReader) {}

    fn on_sample_rejected(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SampleRejectedStatus,
    ) {
    }

    fn on_liveliness_changed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: LivelinessChangedStatus,
    ) {
    }

    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    fn on_publication_matched(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: PublicationMatchedStatus,
    ) {
    }

    fn on_subscription_matched(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SubscriptionMatchedStatus,
    ) {
    }
}
