use crate::{
    dcps::runtime::DdsRuntime,
    publication::publisher_listener::PublisherListener,
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
};

use super::domain_participant_listener::ListenerMail;

pub struct PublisherListenerActor;

impl PublisherListenerActor {
    pub fn spawn<R: DdsRuntime>(
        mut listener: impl PublisherListener<R> + Send + 'static,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<ListenerMail<R>> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
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
                        panic!("Not valid for publisher")
                    }
                    ListenerMail::DataAvailable { the_reader: _ } => {
                        panic!("Not valid for publisher")
                    }
                    ListenerMail::RequestedDeadlineMissed {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for publisher")
                    }
                    ListenerMail::SampleRejected {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for publisher")
                    }
                    ListenerMail::SubscriptionMatched {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for publisher")
                    }
                    ListenerMail::RequestedIncompatibleQos {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for publisher")
                    }
                }
            }
        });
        listener_sender
    }
}
