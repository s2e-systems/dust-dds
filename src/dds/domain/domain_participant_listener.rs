use crate::dds::topic::topic::Topic;
use crate::dds::infrastructure::status::{
    InconsistentTopicStatus,
    LivelinessLostStatus,
    OfferedDeadlineMissedStatus,
    SampleLostStatus,
    SampleRejectedStatus,
    LivelinessChangedStatus,
    RequestedDeadlineMissedStatus,
    RequestedIncompatibleQosStatus,
    PublicationMatchedStatus,
    SubscriptionMatchedStatus};
use crate::dds::subscription::subscriber::Subscriber;
use crate::dds::subscription::data_reader::DataReader;
use crate::dds::publication::data_writer::DataWriter;

/// The purpose of the DomainParticipantListener is to be the listener of last resort that is notified of all status changes not
/// captured by more specific listeners attached to the DomainEntity objects. When a relevant status change occurs, the DCPS
/// Service will first attempt to notify the listener attached to the concerned DomainEntity if one is installed. Otherwise, the
/// DCPS Service will notify the Listener attached to the DomainParticipant.
pub struct DomainParticipantListener{}

impl DomainParticipantListener {
    pub fn on_inconsistent_topic(
        _the_topic: Topic,
        _status: InconsistentTopicStatus,
    ) {
        todo!()
    }

    pub fn on_liveliness_lost(
        _the_writer: DataWriter,
        _status: LivelinessLostStatus,
    ) {
        todo!()
    }

    pub fn on_offered_deadline_missed(
        _the_writer: DataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
        todo!()
    }

    pub fn on_offered_incompatible_qos(
        _the_writer: DataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
        todo!()
    }

    pub fn on_data_on_readers(
        _the_subscriber: Subscriber,
    ) {
        todo!()
    }

    pub fn on_sample_lost(
        _the_reader: DataReader,
        _status: SampleLostStatus,
    ) {
        todo!()
    }

    pub fn on_data_available(
        _the_reader: DataReader,
    ) {
        todo!()
    }

    pub fn on_sample_rejected(
        _the_reader: DataReader,
        _status: SampleRejectedStatus,
    ) {
        todo!()
    }

    pub fn on_liveliness_changed(
        _the_reader: DataReader,
        _status: LivelinessChangedStatus,
    ) {
        todo!()
    }

    pub fn on_requested_deadline_missed(
        _the_reader: DataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
        todo!()
    }

    pub fn on_requested_incompatible_qos(
        _the_reader: DataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
        todo!()
    }

    pub fn on_publication_matched(
        _the_writer: DataWriter,
        _status: PublicationMatchedStatus,
    ) {
        todo!()
    }

    pub fn on_subscription_matched(
        _the_reader: DataReader,
        _status: SubscriptionMatchedStatus,
    ) {
        todo!()
    }
}