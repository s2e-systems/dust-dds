use crate::{
    dds_async::data_writer::DataWriterAsync,
    infrastructure::status::{
        OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
    },
    publication::data_writer_listener::DataWriterListener,
    runtime::{
        executor::ExecutorHandle,
        mpsc::{mpsc_channel, MpscSender},
    },
};

pub struct DataWriterListenerActor;

impl DataWriterListenerActor {
    pub fn spawn<'a, Foo>(
        mut listener: Box<(dyn DataWriterListener<'a, Foo = Foo> + Send + 'a)>,
        executor_handle: &ExecutorHandle,
    ) -> MpscSender<DataWriterListenerMail>
    where
        Foo: 'a,
    {
        let (listener_sender, listener_receiver) = mpsc_channel();
        executor_handle.spawn(async move {
            while let Some(m) = listener_receiver.recv().await {
                match m {
                    DataWriterListenerMail::PublicationMatched { the_writer, status } => {
                        listener
                            .on_publication_matched(the_writer.change_foo_type(), status)
                            .await;
                    }
                    DataWriterListenerMail::OfferedIncompatibleQos { the_writer, status } => {
                        listener
                            .on_offered_incompatible_qos(the_writer.change_foo_type(), status)
                            .await;
                    }
                    DataWriterListenerMail::OfferedDeadlineMissed { the_writer, status } => {
                        listener
                            .on_offered_deadline_missed(the_writer.change_foo_type(), status)
                            .await;
                    }
                }
            }
        });
        listener_sender
    }
}

pub enum DataWriterListenerMail {
    PublicationMatched {
        the_writer: DataWriterAsync<()>,
        status: PublicationMatchedStatus,
    },
    OfferedIncompatibleQos {
        the_writer: DataWriterAsync<()>,
        status: OfferedIncompatibleQosStatus,
    },
    OfferedDeadlineMissed {
        the_writer: DataWriterAsync<()>,
        status: OfferedDeadlineMissedStatus,
    },
}
