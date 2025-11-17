use core::pin::Pin;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    publication::publisher_listener::PublisherListener,
    runtime::{DdsRuntime, Spawner},
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsPublisherListener<R: DdsRuntime> {
    sender: MpscSender<ListenerMail<R>>,
    task: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl<R: DdsRuntime> DcpsPublisherListener<R> {
    pub fn new(mut listener: impl PublisherListener<R> + Send + 'static) -> Self {
        let (sender, listener_receiver) = mpsc_channel();
        let task = Box::pin(async move {
            while let Some(m) = listener_receiver.receive().await {
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
        Self { sender, task }
    }

    pub fn spawn(self, spawner_handle: &R::SpawnerHandle) -> MpscSender<ListenerMail<R>> {
        spawner_handle.spawn(self.task);
        self.sender
    }
}
