use crate::{
    runtime::{ChannelReceive, DdsRuntime, Spawner},
    subscription::subscriber_listener::SubscriberListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct SubscriberListenerActor;

impl SubscriberListenerActor {
    pub fn spawn<R: DdsRuntime>(
        mut listener: impl SubscriberListener<R> + Send + 'static,
        spanwer_handle: &R::SpawnerHandle,
    ) -> R::ChannelSender<ListenerMail<R>> {
        let (listener_sender, mut listener_receiver) = R::channel();
        spanwer_handle.spawn(async move {
            while let Some(m) = listener_receiver.receive().await {
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
