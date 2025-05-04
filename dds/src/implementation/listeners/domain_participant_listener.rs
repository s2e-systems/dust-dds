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
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
};

pub struct DomainParticipantListenerActor;

impl DomainParticipantListenerActor {
    pub fn spawn(
        mut listener: Box<dyn DomainParticipantListenerAsync + Send>,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<DomainParticipantListenerMail> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
                    DomainParticipantListenerMail::RequestedDeadlineMissed {
                        the_reader,
                        status,
                    } => {
                        listener
                            .on_requested_deadline_missed(the_reader, status)
                            .await;
                    }
                    DomainParticipantListenerMail::SampleRejected { the_reader, status } => {
                        listener.on_sample_rejected(the_reader, status).await;
                    }
                    DomainParticipantListenerMail::SubscriptionMatched { the_reader, status } => {
                        listener.on_subscription_matched(the_reader, status).await;
                    }
                    DomainParticipantListenerMail::RequestedIncompatibleQos {
                        the_reader,
                        status,
                    } => {
                        listener
                            .on_requested_incompatible_qos(the_reader, status)
                            .await;
                    }
                    DomainParticipantListenerMail::PublicationMatched { the_writer, status } => {
                        listener.on_publication_matched(the_writer, status).await;
                    }
                    DomainParticipantListenerMail::OfferedIncompatibleQos {
                        the_writer,
                        status,
                    } => {
                        listener
                            .on_offered_incompatible_qos(the_writer, status)
                            .await;
                    }
                    DomainParticipantListenerMail::OfferedDeadlineMissed { the_writer, status } => {
                        listener
                            .on_offered_deadline_missed(the_writer, status)
                            .await;
                    }
                }
            }
        });
        listener_sender
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
