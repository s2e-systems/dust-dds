use super::{
    any_data_reader_listener::{AnyDataReaderListener, DataReaderListenerOperation},
    status_condition_actor::StatusConditionActor,
};
use crate::{
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::{
        actor::ActorAddress,
        runtime::{
            executor::block_on,
            mpsc::{mpsc_channel, MpscSender},
        },
    },
    infrastructure::error::DdsResult,
};
use std::thread::JoinHandle;

pub struct DataReaderListenerMessage {
    listener_operation: DataReaderListenerOperation,
    status_condition_address: ActorAddress<StatusConditionActor>,
    subscriber: SubscriberAsync,
    topic: TopicAsync,
}

pub struct DataReaderListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<DataReaderListenerMessage>,
    subscriber_async: SubscriberAsync,
}

impl DataReaderListenerThread {
    pub fn new(
        mut listener: Box<dyn AnyDataReaderListener + Send>,
        subscriber_async: SubscriberAsync,
    ) -> Self {
        let (sender, receiver) = mpsc_channel::<DataReaderListenerMessage>();
        let thread = std::thread::Builder::new()
            .name("Data reader listener".to_string())
            .spawn(move || {
                block_on(async {
                    while let Some(m) = receiver.recv().await {
                        listener
                            .call_listener_function(
                                m.listener_operation,
                                m.status_condition_address,
                                m.subscriber,
                                m.topic,
                            )
                            .await;
                    }
                });
            })
            .expect("failed to spawn thread");
        Self {
            thread,
            sender,
            subscriber_async,
        }
    }

    fn sender(&self) -> &MpscSender<DataReaderListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}

pub struct DataReaderActorListener {
    pub data_reader_listener: Box<dyn AnyDataReaderListener + Send>,
    pub subscriber_async: SubscriberAsync,
}
