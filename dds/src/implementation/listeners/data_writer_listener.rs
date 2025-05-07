use crate::{
    publication::data_writer_listener::DataWriterListener,
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
};

use super::domain_participant_listener::ListenerMail;

pub struct DataWriterListenerActor;

impl DataWriterListenerActor {
    pub fn spawn<'a, Foo>(
        mut listener: impl DataWriterListener<'a, Foo> + Send + 'static,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<ListenerMail>
    where
        Foo: 'a,
    {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
                    ListenerMail::PublicationMatched { the_writer, status } => {
                        listener
                            .on_publication_matched(the_writer.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::OfferedIncompatibleQos { the_writer, status } => {
                        listener
                            .on_offered_incompatible_qos(the_writer.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::OfferedDeadlineMissed { the_writer, status } => {
                        listener
                            .on_offered_deadline_missed(the_writer.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::DataAvailable { the_reader: _ } => {
                        panic!("Not valid for writer")
                    }
                    ListenerMail::DataOnReaders { the_subscriber: _ } => {
                        panic!("Not valid for writer")
                    }
                    ListenerMail::RequestedDeadlineMissed {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for writer")
                    }
                    ListenerMail::SampleRejected {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for writer")
                    }
                    ListenerMail::SubscriptionMatched {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for writer")
                    }
                    ListenerMail::RequestedIncompatibleQos {
                        the_reader: _,
                        status: _,
                    } => {
                        panic!("Not valid for writer")
                    }
                }
            }
        });
        listener_sender
    }
}
