use core::pin::Pin;

use crate::{
    dcps::channels::mpsc::{MpscSender, mpsc_channel},
    runtime::DdsRuntime,
    subscription::subscriber_listener::SubscriberListener,
};

use super::domain_participant_listener::ListenerMail;

pub struct DcpsSubscriberListener;

impl DcpsSubscriberListener {
    pub fn spawn<R: DdsRuntime>(
        mut listener: impl SubscriberListener<R> + Send + 'static,
    ) -> (
        MpscSender<ListenerMail<R>>,
        Pin<Box<dyn Future<Output = ()> + Send>>,
    ) {
        let (listener_sender, listener_receiver) = mpsc_channel();
        let listener_task = Box::pin(async move {
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
        (listener_sender, listener_task)
    }
}
