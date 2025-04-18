use crate::{
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync,
        domain_participant_listener::DomainParticipantListenerAsync,
    },
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::{actor::MailHandler, executor::block_on},
};

pub struct DomainParticipantListenerActor {
    listener: Box<dyn DomainParticipantListenerAsync + Send>,
}

impl DomainParticipantListenerActor {
    pub fn new(listener: Box<dyn DomainParticipantListenerAsync + Send>) -> Self {
        Self { listener }
    }
}

pub struct TriggerRequestedDeadlineMissed {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedDeadlineMissedStatus,
}
impl MailHandler<TriggerRequestedDeadlineMissed> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerRequestedDeadlineMissed) {
        block_on(
            self.listener
                .on_requested_deadline_missed(message.the_reader, message.status),
        );
    }
}

pub struct TriggerSampleRejected {
    pub the_reader: DataReaderAsync<()>,
    pub status: SampleRejectedStatus,
}
impl MailHandler<TriggerSampleRejected> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerSampleRejected) {
        block_on(
            self.listener
                .on_sample_rejected(message.the_reader, message.status),
        );
    }
}

pub struct TriggerSubscriptionMatched {
    pub the_reader: DataReaderAsync<()>,
    pub status: SubscriptionMatchedStatus,
}
impl MailHandler<TriggerSubscriptionMatched> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerSubscriptionMatched) {
        block_on(
            self.listener
                .on_subscription_matched(message.the_reader, message.status),
        )
    }
}

pub struct TriggerRequestedIncompatibleQos {
    pub the_reader: DataReaderAsync<()>,
    pub status: RequestedIncompatibleQosStatus,
}
impl MailHandler<TriggerRequestedIncompatibleQos> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerRequestedIncompatibleQos) {
        block_on(
            self.listener
                .on_requested_incompatible_qos(message.the_reader, message.status),
        )
    }
}

pub struct TriggerPublicationMatched {
    pub the_writer: DataWriterAsync<()>,
    pub status: PublicationMatchedStatus,
}
impl MailHandler<TriggerPublicationMatched> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerPublicationMatched) {
        block_on(
            self.listener
                .on_publication_matched(message.the_writer, message.status),
        )
    }
}

pub struct TriggerOfferedIncompatibleQos {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedIncompatibleQosStatus,
}
impl MailHandler<TriggerOfferedIncompatibleQos> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerOfferedIncompatibleQos) {
        block_on(
            self.listener
                .on_offered_incompatible_qos(message.the_writer, message.status),
        )
    }
}

pub struct TriggerOfferedDeadlineMissed {
    pub the_writer: DataWriterAsync<()>,
    pub status: OfferedDeadlineMissedStatus,
}
impl MailHandler<TriggerOfferedDeadlineMissed> for DomainParticipantListenerActor {
    fn handle(&mut self, message: TriggerOfferedDeadlineMissed) {
        block_on(
            self.listener
                .on_offered_deadline_missed(message.the_writer, message.status),
        )
    }
}
