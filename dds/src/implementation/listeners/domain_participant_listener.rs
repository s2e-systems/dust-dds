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
    RequestedDeadlineMissed {
        the_reader: DataReaderAsync<()>,
        status: RequestedDeadlineMissedStatus,
    },
    SampleRejected {
        the_reader: DataReaderAsync<()>,
        status: SampleRejectedStatus,
    },
    SubscriptionMatched {
        the_reader: DataReaderAsync<()>,
        status: SubscriptionMatchedStatus,
    },
    RequestedIncompatibleQos {
        the_reader: DataReaderAsync<()>,
        status: RequestedIncompatibleQosStatus,
    },
    PublicationMatched {
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    },
    OfferedIncompatibleQos {
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    },
    OfferedDeadlineMissed {
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    },
}

impl MailHandler for DomainParticipantListenerActor {
    type Mail = DomainParticipantListenerMail;
    fn handle(&mut self, message: DomainParticipantListenerMail) {
        match message {
            DomainParticipantListenerMail::RequestedDeadlineMissed { the_reader, status } => {
                block_on(
                    self.listener
                        .on_requested_deadline_missed(the_reader, status),
                )
            }
            DomainParticipantListenerMail::SampleRejected { the_reader, status } => {
                block_on(self.listener.on_sample_rejected(the_reader, status))
            }
            DomainParticipantListenerMail::SubscriptionMatched { the_reader, status } => {
                block_on(self.listener.on_subscription_matched(the_reader, status))
            }
            DomainParticipantListenerMail::RequestedIncompatibleQos { the_reader, status } => {
                block_on(
                    self.listener
                        .on_requested_incompatible_qos(the_reader, status),
                )
            }
            DomainParticipantListenerMail::PublicationMatched { the_writer, status } => {
                block_on(self.listener.on_publication_matched(the_writer, status))
            }
            DomainParticipantListenerMail::OfferedIncompatibleQos { the_writer, status } => {
                block_on(
                    self.listener
                        .on_offered_incompatible_qos(the_writer, status),
                )
            }
            DomainParticipantListenerMail::OfferedDeadlineMissed { the_writer, status } => {
                block_on(self.listener.on_offered_deadline_missed(the_writer, status))
            }
        }
    }
}
