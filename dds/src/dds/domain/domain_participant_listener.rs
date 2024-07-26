use std::{future::Future, pin::Pin};

use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant_listener::DomainParticipantListenerAsync, topic::TopicAsync,
    },
    infrastructure::status::{
        InconsistentTopicStatus, LivelinessChangedStatus, LivelinessLostStatus,
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleLostStatus,
        SampleRejectedStatus, SubscriptionMatchedStatus,
    },
    publication::data_writer::DataWriter,
    subscription::data_reader::DataReader,
    topic_definition::topic::Topic,
};

/// The purpose of the DomainParticipantListener is to be the listener of last resort that is notified of all status changes not
/// captured by more specific listeners attached to the DomainEntity objects. When a relevant status change occurs, the DCPS
/// Service will first attempt to notify the listener attached to the concerned DomainEntity if one is installed. Otherwise, the
/// DCPS Service will notify the Listener attached to the DomainParticipant.
pub trait DomainParticipantListener {
    /// Method that is called when any inconsistent topic is discovered in the domain participant.
    fn on_inconsistent_topic(&mut self, _the_topic: Topic, _status: InconsistentTopicStatus) {}

    /// Method that is called when any writer in the domain participant reports a liveliness lost status.
    fn on_liveliness_lost(&mut self, _the_writer: DataWriter<()>, _status: LivelinessLostStatus) {}

    /// Method that is called when any data writer in the domain participant reports a deadline missed status.
    fn on_offered_deadline_missed(
        &mut self,
        _the_writer: DataWriter<()>,
        _status: OfferedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any data writer in the domain participant reports an offered incompatible QoS status.
    fn on_offered_incompatible_qos(
        &mut self,
        _the_writer: DataWriter<()>,
        _status: OfferedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a sample lost status.
    fn on_sample_lost(&mut self, _the_reader: DataReader<()>, _status: SampleLostStatus) {}

    /// Method that is called when any data reader in the domain participant reports a data available status.
    fn on_data_available(&mut self, _the_reader: DataReader<()>) {}

    /// Method that is called when any data reader in the domain participant reports a sample rejected status.
    fn on_sample_rejected(&mut self, _the_reader: DataReader<()>, _status: SampleRejectedStatus) {}

    /// Method that is called when any data reader in the domain participant reports a liveliness changed status.
    fn on_liveliness_changed(
        &mut self,
        _the_reader: DataReader<()>,
        _status: LivelinessChangedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a requested deadline missed status.
    fn on_requested_deadline_missed(
        &mut self,
        _the_reader: DataReader<()>,
        _status: RequestedDeadlineMissedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a requested incompatible QoS status.
    fn on_requested_incompatible_qos(
        &mut self,
        _the_reader: DataReader<()>,
        _status: RequestedIncompatibleQosStatus,
    ) {
    }

    /// Method that is called when any data writer in the domain participant reports a publication matched status.
    fn on_publication_matched(
        &mut self,
        _the_writer: DataWriter<()>,
        _status: PublicationMatchedStatus,
    ) {
    }

    /// Method that is called when any data reader in the domain participant reports a subscription matched status.
    fn on_subscription_matched(
        &mut self,
        _the_reader: DataReader<()>,
        _status: SubscriptionMatchedStatus,
    ) {
    }
}

impl DomainParticipantListenerAsync for Box<dyn DomainParticipantListener + Send> {
    fn on_inconsistent_topic(
        &mut self,
        the_topic: TopicAsync,
        status: InconsistentTopicStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_inconsistent_topic(
            self.as_mut(),
            Topic::new(the_topic),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_liveliness_lost(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: LivelinessLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_liveliness_lost(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_offered_deadline_missed(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_offered_deadline_missed(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_offered_incompatible_qos(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_offered_incompatible_qos(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_sample_lost(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SampleLostStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_sample_lost(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_data_available(
        &mut self,
        the_reader: DataReaderAsync<()>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_data_available(self.as_mut(), DataReader::new(the_reader));
        Box::pin(std::future::ready(()))
    }

    fn on_sample_rejected(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_sample_rejected(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_liveliness_changed(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: LivelinessChangedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_liveliness_changed(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_requested_deadline_missed(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_requested_deadline_missed(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_requested_incompatible_qos(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_requested_incompatible_qos(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_publication_matched(
        &mut self,
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_publication_matched(
            self.as_mut(),
            DataWriter::new(the_writer),
            status,
        );
        Box::pin(std::future::ready(()))
    }

    fn on_subscription_matched(
        &mut self,
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        DomainParticipantListener::on_subscription_matched(
            self.as_mut(),
            DataReader::new(the_reader),
            status,
        );
        Box::pin(std::future::ready(()))
    }
}
