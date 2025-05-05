use crate::{
    dds_async::{data_reader::DataReaderAsync, subscriber::SubscriberAsync},
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
    subscription::subscriber_listener::SubscriberListener,
};

pub struct SubscriberListenerActor;

impl SubscriberListenerActor {
    pub fn spawn(
        mut listener: Box<dyn SubscriberListener + Send>,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<SubscriberListenerMail> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
                    SubscriberListenerMail::DataOnReaders { the_subscriber } => {
                        listener.on_data_on_readers(the_subscriber).await;
                    }
                    SubscriberListenerMail::RequestedDeadlineMissed { the_reader, status } => {
                        listener
                            .on_requested_deadline_missed(the_reader, status)
                            .await;
                    }
                    SubscriberListenerMail::SampleRejected { the_reader, status } => {
                        listener.on_sample_rejected(the_reader, status).await;
                    }
                    SubscriberListenerMail::SubscriptionMatched { the_reader, status } => {
                        listener.on_subscription_matched(the_reader, status).await;
                    }
                    SubscriberListenerMail::RequestedIncompatibleQos { the_reader, status } => {
                        listener
                            .on_requested_incompatible_qos(the_reader, status)
                            .await;
                    }
                }
            }
        });
        listener_sender
    }
}

pub enum SubscriberListenerMail {
    DataOnReaders {
        the_subscriber: SubscriberAsync,
    },
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
}
