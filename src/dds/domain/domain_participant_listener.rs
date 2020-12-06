// use crate::topic::Topic;
use crate::infrastructure::status::{
    // InconsistentTopicStatus,
    LivelinessLostStatus,
    OfferedDeadlineMissedStatus,
    SampleLostStatus,
    SampleRejectedStatus,
    LivelinessChangedStatus,
    RequestedDeadlineMissedStatus,
    RequestedIncompatibleQosStatus,
    PublicationMatchedStatus,
    SubscriptionMatchedStatus};
use crate::subscription::{Subscriber, SubscriberListener, AnyDataReader};
// use crate::publication::AnyDataWriter;
use crate::infrastructure::qos::SubscriberQos;


/// The purpose of the DomainParticipantListener is to be the listener of last resort that is notified of all status changes not
/// captured by more specific listeners attached to the DomainEntity objects. When a relevant status change occurs, the DCPS
/// Service will first attempt to notify the listener attached to the concerned DomainEntity if one is installed. Otherwise, the
/// DCPS Service will notify the Listener attached to the DomainParticipant.
pub trait DomainParticipantListener: 'static{
    // fn on_inconsistent_topic(&self, _the_topic: Topic, _status: InconsistentTopicStatus);

    // fn on_liveliness_lost(
    //     &self,
    //     _the_writer: &dyn AnyDataWriter,
    //     _status: LivelinessLostStatus,
    // );

    // fn on_offered_deadline_missed(
    //     &self,
    //     _the_writer: &dyn AnyDataWriter,
    //     _status: OfferedDeadlineMissedStatus,
    // );

    // fn on_offered_incompatible_qos(
    //     &self,
    //     _the_writer: &dyn AnyDataWriter,
    //     _status: OfferedDeadlineMissedStatus,
    // );

    // fn on_data_on_readers(
    //     &self,
    //     _the_subscriber: &dyn Subscriber,
    // );

    // fn on_sample_lost(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    //     _status: SampleLostStatus,
    // );

    // fn on_data_available(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    // );

    // fn on_sample_rejected(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    //     _status: SampleRejectedStatus,
    // );

    // fn on_liveliness_changed(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    //     _status: LivelinessChangedStatus,
    // );

    // fn on_requested_deadline_missed(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    //     _status: RequestedDeadlineMissedStatus,
    // );

    // fn on_requested_incompatible_qos(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    //     _status: RequestedIncompatibleQosStatus,
    // );

    // fn on_publication_matched(
    //     &self,
    //     _the_writer: &dyn AnyDataWriter,
    //     _status: PublicationMatchedStatus,
    // );

    // fn on_subscription_matched(
    //     &self,
    //     _the_reader: &dyn AnyDataReader,
    //     _status: SubscriptionMatchedStatus,
    // );
}