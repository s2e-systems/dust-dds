use core::pin::Pin;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    publication::data_writer_listener::DataWriterListener,
    runtime::DdsRuntime,
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsDataWriterListener;

impl DcpsDataWriterListener {
    pub fn spawn<R: DdsRuntime, Foo>(
        mut listener: impl DataWriterListener<R, Foo> + Send + 'static,
    ) -> (
        MpscSender<ListenerMail<R>>,
        Pin<Box<dyn Future<Output = ()> + Send>>,
    ) {
        let (listener_sender, listener_receiver) = mpsc_channel();
        let listener_future = Box::pin(async move {
            while let Some(m) = listener_receiver.receive().await {
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
        (listener_sender, listener_future)
    }
}
