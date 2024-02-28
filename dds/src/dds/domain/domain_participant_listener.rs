use crate::{
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::data_writer::AnyDataWriter,
    subscription::data_reader::AnyDataReader,
    topic_definition::topic::Topic,
};

/// The purpose of the DomainParticipantListener is to be the listener of last resort that is notified of all status changes not
/// captured by more specific listeners attached to the DomainEntity objects. When a relevant status change occurs, the DCPS
/// Service will first attempt to notify the listener attached to the concerned DomainEntity if one is installed. Otherwise, the
/// DCPS Service will notify the Listener attached to the DomainParticipant.
pub trait DomainParticipantListener {
    /// Method that is called when any inconsistent topic is discovered in the domain participant.
    fn on_inconsistent_topic(&mut self, _the_topic: &Topic, _status: InconsistentTopicStatus) {}

    /// Method that is called when any writer in the domain participant reports a liveliness lost status.
    fn on_liveliness_lost(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: LivelinessLostStatus,
    ) {
    }

    /// Method that is called when any data writer in the domain participant reports a deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any data writer in the domain participant reports an offered incompatible QoS status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a sample lost status.
    fn on_sample_lost(&mut self, _the_reader: &dyn AnyDataReader, _status: SampleLostStatus) {}

    /// Method that is called when any data reader in the domain participant reports a data available status.
    fn on_data_available(&mut self, _the_reader: &dyn AnyDataReader) {}

    /// Method that is called when any data reader in the domain participant reports a sample rejected status.
    fn on_sample_rejected(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SampleRejectedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: LivelinessChangedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any data writer in the domain participant reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: &dyn AnyDataWriter,
        _status: PublicationMatchedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: &dyn AnyDataReader,
        _status: SubscriptionMatchedStatus,
    ) {
    }
}
