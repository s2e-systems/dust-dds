use tracing::span;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    runtime::{DdsRuntime, Spawner},
    subscription::data_reader_listener::DataReaderListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsDataReaderListener;

impl DcpsDataReaderListener {
    #[tracing::instrument(skip(listener, spawner_handle))]
    pub fn spawn<R: DdsRuntime, Foo>(
        mut listener: impl DataReaderListener<R, Foo> + Send + 'static,
        spawner_handle: &R::SpawnerHandle,
    ) -> MpscSender<ListenerMail<R>> {
        let (listener_sender, listener_receiver) = mpsc_channel();
        spawner_handle.spawn(async move {
            while let Some(m) = listener_receiver.receive().await {
                let listener_root = span!(tracing::Level::INFO, "data_reader_listener_triggered");
                let _listener_span_guard = listener_root.enter();
                match m {
                    ListenerMail::DataAvailable { the_reader } => {
                        let root = span!(tracing::Level::INFO, "on_data_available");
                        let _guard = root.enter();
                        listener
                            .on_data_available(the_reader.change_foo_type())
                            .await;
                    }
                    ListenerMail::RequestedDeadlineMissed { the_reader, status } => {
                        listener
                            .on_requested_deadline_missed(the_reader.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::SampleRejected { the_reader, status } => {
                        listener
                            .on_sample_rejected(the_reader.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::SubscriptionMatched { the_reader, status } => {
                        listener
                            .on_subscription_matched(the_reader.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::RequestedIncompatibleQos { the_reader, status } => {
                        listener
                            .on_requested_incompatible_qos(the_reader.change_foo_type(), status)
                            .await;
                    }
                    ListenerMail::DataOnReaders { the_subscriber: _ } => {
                        panic!("Not valid for reader")
                    }
                    ListenerMail::PublicationMatched {
                        the_writer: _,
                        status: _,
                    } => {
                        panic!("Not valid for reader")
                    }
                    ListenerMail::OfferedIncompatibleQos {
                        the_writer: _,
                        status: _,
                    } => {
                        panic!("Not valid for reader")
                    }
                    ListenerMail::OfferedDeadlineMissed {
                        the_writer: _,
                        status: _,
                    } => {
                        panic!("Not valid for reader")
                    }
                }
            }
        });
        listener_sender
    }
}
