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

pub enum DomainParticipantListenerMail {
    TriggerRequestedDeadlineMissed {
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    },
    TriggerSampleRejected {
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    },
    TriggerSubscriptionMatched {
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    },
    TriggerRequestedIncompatibleQos {
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    },
    TriggerPublicationMatched {
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    },
    TriggerOfferedIncompatibleQos {
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    },
    TriggerOfferedDeadlineMissed {
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    },
}

impl MailHandler<DomainParticipantListenerMail> for DomainParticipantListenerActor {
    fn handle(&mut self, message: DomainParticipantListenerMail) {
        match message {
            DomainParticipantListenerMail::TriggerRequestedDeadlineMissed {
                the_reader,
                status,
            } => block_on(
                self.listener
                    .on_requested_deadline_missed(the_reader, status),
            ),
            DomainParticipantListenerMail::TriggerSampleRejected { the_reader, status } => {
                block_on(self.listener.on_sample_rejected(the_reader, status))
            }
            DomainParticipantListenerMail::TriggerSubscriptionMatched { the_reader, status } => {
                block_on(self.listener.on_subscription_matched(the_reader, status))
            }
            DomainParticipantListenerMail::TriggerRequestedIncompatibleQos {
                the_reader,
                status,
            } => block_on(
                self.listener
                    .on_requested_incompatible_qos(the_reader, status),
            ),
            DomainParticipantListenerMail::TriggerPublicationMatched { the_writer, status } => {
                block_on(self.listener.on_publication_matched(the_writer, status))
            }
            DomainParticipantListenerMail::TriggerOfferedIncompatibleQos { the_writer, status } => {
                block_on(
                    self.listener
                        .on_offered_incompatible_qos(the_writer, status),
                )
            }
            DomainParticipantListenerMail::TriggerOfferedDeadlineMissed { the_writer, status } => {
                block_on(self.listener.on_offered_deadline_missed(the_writer, status))
            }
        }
    }
}
