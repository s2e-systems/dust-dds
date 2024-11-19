use crate::{
    dds_async::{
        subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync,
        topic::TopicAsync,
    },
    implementation::{
        actor::ActorAddress, runtime::mpsc::MpscSender,
        status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, SubscriptionMatchedStatus,
        },
    },
};

use std::thread::JoinHandle;

pub enum SubscriberListenerOperation {
    DataOnReaders(SubscriberAsync),
    _DataAvailable,
    SampleRejected(SampleRejectedStatus),
    _LivenessChanged(LivelinessChangedStatus),
    RequestedDeadlineMissed(RequestedDeadlineMissedStatus),
    RequestedIncompatibleQos(RequestedIncompatibleQosStatus),
    SubscriptionMatched(SubscriptionMatchedStatus),
    SampleLost(SampleLostStatus),
}

pub struct SubscriberListenerMessage {
    pub listener_operation: SubscriberListenerOperation,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub subscriber: SubscriberAsync,
    pub topic: TopicAsync,
}

pub struct SubscriberListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<SubscriberListenerMessage>,
}

impl SubscriberListenerThread {
    pub fn new(mut listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
        // let (sender, receiver) = mpsc_channel::<SubscriberListenerMessage>();
        // let thread = std::thread::Builder::new()
        //     .name("Subscriber listener".to_string())
        //     .spawn(move || {
        //         block_on(async {
        //             while let Some(m) = receiver.recv().await {
        //                 let data_reader = DataReaderAsync::new(
        //                     m.reader_address,
        //                     m.status_condition_address,
        //                     m.subscriber,
        //                     m.topic,
        //                 );
        //                 match m.listener_operation {
        //                     SubscriberListenerOperation::DataOnReaders(the_subscriber) => {
        //                         listener.on_data_on_readers(the_subscriber).await
        //                     }
        //                     SubscriberListenerOperation::_DataAvailable => {
        //                         listener.on_data_available(data_reader).await
        //                     }
        //                     SubscriberListenerOperation::SampleRejected(status) => {
        //                         listener.on_sample_rejected(data_reader, status).await
        //                     }
        //                     SubscriberListenerOperation::_LivenessChanged(status) => {
        //                         listener.on_liveliness_changed(data_reader, status).await
        //                     }
        //                     SubscriberListenerOperation::RequestedDeadlineMissed(status) => {
        //                         listener
        //                             .on_requested_deadline_missed(data_reader, status)
        //                             .await
        //                     }
        //                     SubscriberListenerOperation::RequestedIncompatibleQos(status) => {
        //                         listener
        //                             .on_requested_incompatible_qos(data_reader, status)
        //                             .await
        //                     }
        //                     SubscriberListenerOperation::SubscriptionMatched(status) => {
        //                         listener.on_subscription_matched(data_reader, status).await
        //                     }
        //                     SubscriberListenerOperation::SampleLost(status) => {
        //                         listener.on_sample_lost(data_reader, status).await
        //                     }
        //                 }
        //             }
        //         });
        //     })
        //     .expect("failed to spawn thread");
        // Self { thread, sender }
        todo!()
    }

    fn sender(&self) -> &MpscSender<SubscriberListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}
