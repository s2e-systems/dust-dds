use crate::{
    runtime::{ChannelReceive, DdsRuntime, Spawner},
    dds_async::{
        data_reader::DataReaderAsync, data_writer::DataWriterAsync, subscriber::SubscriberAsync,
    },
    domain::domain_participant_listener::DomainParticipantListener,
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
};

pub struct DomainParticipantListenerActor;

impl DomainParticipantListenerActor {
    pub fn spawn<R: DdsRuntime>(
        mut listener: impl DomainParticipantListener<R> + Send + 'static,
        spawner_handle: &R::SpawnerHandle,
    ) -> R::ChannelSender<ListenerMail<R>> {
        let (listener_sender, mut listener_receiver) = R::channel();
        spawner_handle.spawn(async move {
            while let Some(m) = listener_receiver.receive().await {
                match m {
                    ListenerMail::DataAvailable { the_reader } => {
                        listener.on_data_available(the_reader).await;
                    }
                    ListenerMail::RequestedDeadlineMissed { the_reader, status } => {
                        listener
                            .on_requested_deadline_missed(the_reader, status)
                            .await;
                    }
                    ListenerMail::SampleRejected { the_reader, status } => {
                        listener.on_sample_rejected(the_reader, status).await;
                    }
                    ListenerMail::SubscriptionMatched { the_reader, status } => {
                        listener.on_subscription_matched(the_reader, status).await;
                    }
                    ListenerMail::RequestedIncompatibleQos { the_reader, status } => {
                        listener
                            .on_requested_incompatible_qos(the_reader, status)
                            .await;
                    }
                    ListenerMail::PublicationMatched { the_writer, status } => {
                        listener.on_publication_matched(the_writer, status).await;
                    }
                    ListenerMail::OfferedIncompatibleQos { the_writer, status } => {
                        listener
                            .on_offered_incompatible_qos(the_writer, status)
                            .await;
                    }
                    ListenerMail::OfferedDeadlineMissed { the_writer, status } => {
                        listener
                            .on_offered_deadline_missed(the_writer, status)
                            .await;
                    }
                    ListenerMail::DataOnReaders { the_subscriber: _ } => {
                        panic!("Not valid for domain participant")
                    }
                }
            }
        });
        listener_sender
    }
}

pub enum ListenerMail<R: DdsRuntime> {
    DataAvailable {
        the_reader: DataReaderAsync<R, ()>,
    },
    DataOnReaders {
        the_subscriber: SubscriberAsync<R>,
    },
    RequestedDeadlineMissed {
        the_reader: DataReaderAsync<R, ()>,
        status: RequestedDeadlineMissedStatus,
    },
    SampleRejected {
        the_reader: DataReaderAsync<R, ()>,
        status: SampleRejectedStatus,
    },
    SubscriptionMatched {
        the_reader: DataReaderAsync<R, ()>,
        status: SubscriptionMatchedStatus,
    },
    RequestedIncompatibleQos {
        the_reader: DataReaderAsync<R, ()>,
        status: RequestedIncompatibleQosStatus,
    },
    PublicationMatched {
        the_writer: DataWriterAsync<R, ()>,
        status: PublicationMatchedStatus,
    },
    OfferedIncompatibleQos {
        the_writer: DataWriterAsync<R, ()>,
        status: OfferedIncompatibleQosStatus,
    },
    OfferedDeadlineMissed {
        the_writer: DataWriterAsync<R, ()>,
        status: OfferedDeadlineMissedStatus,
    },
}
