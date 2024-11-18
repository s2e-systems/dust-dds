use std::thread::JoinHandle;

use crate::{
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::{
        actor::ActorAddress,
        status_condition::status_condition_actor::StatusConditionActor,
        runtime::{
            executor::block_on,
            mpsc::{mpsc_channel, MpscSender},
        },
    },
    infrastructure::error::DdsResult,
};

use super::{
    any_data_writer_listener::{AnyDataWriterListener, DataWriterListenerOperation},
    data_writer::DataWriterActor,
};

pub struct DataWriterListenerMessage {
    listener_operation: DataWriterListenerOperation,
    writer_address: ActorAddress<DataWriterActor>,
    status_condition_address: ActorAddress<StatusConditionActor>,
    publisher: PublisherAsync,
    topic: TopicAsync,
}

pub struct DataWriterListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<DataWriterListenerMessage>,
}

impl DataWriterListenerThread {
    pub fn new(mut listener: Box<dyn AnyDataWriterListener + Send>) -> Self {
        let (sender, receiver) = mpsc_channel::<DataWriterListenerMessage>();
        let thread = std::thread::Builder::new()
            .name("Data writer listener".to_string())
            .spawn(move || {
                block_on(async {
                    while let Some(m) = receiver.recv().await {
                        listener
                            .call_listener_function(
                                m.listener_operation,
                                m.writer_address,
                                m.status_condition_address,
                                m.publisher,
                                m.topic,
                            )
                            .await;
                    }
                });
            })
            .expect("failed to spawn thread");
        Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<DataWriterListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}
