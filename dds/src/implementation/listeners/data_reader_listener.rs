use crate::{
    dds_async::data_reader::DataReaderAsync,
    infrastructure::status::{
        RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus, SampleRejectedStatus,
        SubscriptionMatchedStatus,
    },
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
    subscription::data_reader_listener::DataReaderListener,
};

pub struct DataReaderListenerActor;

impl DataReaderListenerActor {
    pub fn spawn<'a, Foo>(
        mut listener: impl DataReaderListener<'a, Foo> + Send + 'static,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<DataReaderListenerMail>
    where
        Foo: 'a,
    {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
                    DataReaderListenerMail::DataAvailable { the_reader } => {
                        listener
                            .on_data_available(the_reader.change_foo_type())
                            .await;
                    }
                    DataReaderListenerMail::RequestedDeadlineMissed { the_reader, status } => {
                        listener
                            .on_requested_deadline_missed(the_reader.change_foo_type(), status)
                            .await;
                    }
                    DataReaderListenerMail::SampleRejected { the_reader, status } => {
                        listener
                            .on_sample_rejected(the_reader.change_foo_type(), status)
                            .await;
                    }
                    DataReaderListenerMail::SubscriptionMatched { the_reader, status } => {
                        listener
                            .on_subscription_matched(the_reader.change_foo_type(), status)
                            .await;
                    }
                    DataReaderListenerMail::RequestedIncompatibleQos { the_reader, status } => {
                        listener
                            .on_requested_incompatible_qos(the_reader.change_foo_type(), status)
                            .await;
                    }
                }
            }
        });
        listener_sender
    }
}

pub enum DataReaderListenerMail {
    DataAvailable {
        the_reader: DataReaderAsync<()>,
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
