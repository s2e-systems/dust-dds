use crate::{
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
    subscription::subscriber_listener::SubscriberListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct SubscriberListenerActor;

impl SubscriberListenerActor {
    pub fn spawn(
        mut listener: impl SubscriberListener + Send + 'static,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<ListenerMail> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
                    ListenerMail::DataOnReaders { the_subscriber } => {
                        listener.on_data_on_readers(the_subscriber).await;
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
                    ListenerMail::DataAvailable { the_reader: _ } => {
                        panic!("Not valid for subscriber")
                    }
                    ListenerMail::PublicationMatched {
                        the_writer: _,
                        status: _,
                    } => {
                        panic!("Not valid for subscriber")
                    }
                    ListenerMail::OfferedIncompatibleQos {
                        the_writer: _,
                        status: _,
                    } => {
                        panic!("Not valid for subscriber")
                    }
                    ListenerMail::OfferedDeadlineMissed {
                        the_writer: _,
                        status: _,
                    } => {
                        panic!("Not valid for subscriber")
                    }
                }
            }
        });
        listener_sender
    }
}
