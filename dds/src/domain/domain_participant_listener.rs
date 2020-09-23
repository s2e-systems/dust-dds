use std::any::Any;

use crate::topic::topic::Topic;
use crate::infrastructure::status::{
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
use crate::subscription::subscriber::Subscriber;
use crate::subscription::data_reader::AnyDataReader;
use crate::publication::data_writer::AnyDataWriter;
use crate::infrastructure::listener::NoListener;

/// The purpose of the DomainParticipantListener is to be the listener of last resort that is notified of all status changes not
/// captured by more specific listeners attached to the DomainEntity objects. When a relevant status change occurs, the DCPS
/// Service will first attempt to notify the listener attached to the concerned DomainEntity if one is installed. Otherwise, the
/// DCPS Service will notify the Listener attached to the DomainParticipant.
pub trait DomainParticipantListener : Any + Send + Sync{
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus);

    fn on_liveliness_lost(
        &self,
        _the_writer: AnyDataWriter,
        _status: LivelinessLostStatus,
    );

    fn on_offered_deadline_missed(
        &self,
        _the_writer: AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    );

    fn on_offered_incompatible_qos(
        &self,
        _the_writer: AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    );

    fn on_data_on_readers(
        &self,
        _the_subscriber: Subscriber,
    );

    fn on_sample_lost(
        &self,
        _the_reader: AnyDataReader,
        _status: SampleLostStatus,
    );

    fn on_data_available(
        &self,
        _the_reader: AnyDataReader,
    );

    fn on_sample_rejected(
        &self,
        _the_reader: AnyDataReader,
        _status: SampleRejectedStatus,
    );

    fn on_liveliness_changed(
        &self,
        _the_reader: AnyDataReader,
        _status: LivelinessChangedStatus,
    );

    fn on_requested_deadline_missed(
        &self,
        _the_reader: AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    );

    fn on_requested_incompatible_qos(
        &self,
        _the_reader: AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    );

    fn on_publication_matched(
        &self,
        _the_writer: AnyDataWriter,
        _status: PublicationMatchedStatus,
    );

    fn on_subscription_matched(
        &self,
        _the_reader: AnyDataReader,
        _status: SubscriptionMatchedStatus,
    );
}

impl DomainParticipantListener for NoListener {
    fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus) {
        todo!()
    }

    fn on_liveliness_lost(
        &self,
        _the_writer: AnyDataWriter,
        _status: LivelinessLostStatus,
    ) {
        todo!()
    }

    fn on_offered_deadline_missed(
        &self,
        _the_writer: AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
        todo!()
    }

    fn on_offered_incompatible_qos(
        &self,
        _the_writer: AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
        todo!()
    }

    fn on_data_on_readers(
        &self,
        _the_subscriber: Subscriber,
    ) {
        todo!()
    }

    fn on_sample_lost(
        &self,
        _the_reader: AnyDataReader,
        _status: SampleLostStatus,
    ) {
        todo!()
    }

    fn on_data_available(
        &self,
        _the_reader: AnyDataReader,
    ) {
        todo!()
    }

    fn on_sample_rejected(
        &self,
        _the_reader: AnyDataReader,
        _status: SampleRejectedStatus,
    ) {
        todo!()
    }

    fn on_liveliness_changed(
        &self,
        _the_reader: AnyDataReader,
        _status: LivelinessChangedStatus,
    ) {
        todo!()
    }

    fn on_requested_deadline_missed(
        &self,
        _the_reader: AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
        todo!()
    }

    fn on_requested_incompatible_qos(
        &self,
        _the_reader: AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
        todo!()
    }

    fn on_publication_matched(
        &self,
        _the_writer: AnyDataWriter,
        _status: PublicationMatchedStatus,
    ) {
        todo!()
    }

    fn on_subscription_matched(
        &self,
        _the_reader: AnyDataReader,
        _status: SubscriptionMatchedStatus,
    ) {
        todo!()
    }
}