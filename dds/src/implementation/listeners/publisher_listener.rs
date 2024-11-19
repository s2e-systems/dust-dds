use crate::{
    dds_async::{
        publisher::PublisherAsync, publisher_listener::PublisherListenerAsync, topic::TopicAsync,
    },
    implementation::{
        actor::ActorAddress, domain_participant_backend::entities::data_writer::DataWriterActor,
        runtime::mpsc::MpscSender, status_condition::status_condition_actor::StatusConditionActor,
    },
    infrastructure::{
        error::DdsResult,
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus,
        },
    },
};

use std::thread::JoinHandle;

pub enum PublisherListenerOperation {
    _LivelinessLost(LivelinessLostStatus),
    OfferedDeadlineMissed(OfferedDeadlineMissedStatus),
    OfferedIncompatibleQos(OfferedIncompatibleQosStatus),
    PublicationMatched(PublicationMatchedStatus),
}

pub struct PublisherListenerMessage {
    pub listener_operation: PublisherListenerOperation,
    pub writer_address: ActorAddress<DataWriterActor>,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub publisher: PublisherAsync,
    pub topic: TopicAsync,
}

pub struct PublisherListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<PublisherListenerMessage>,
}

impl PublisherListenerThread {
    pub fn new(mut listener: Box<dyn PublisherListenerAsync + Send>) -> Self {
        todo!()
        // let (sender, receiver) = mpsc_channel::<PublisherListenerMessage>();
        // let thread = std::thread::Builder::new()
        //     .name("Publisher listener".to_string())
        //     .spawn(move || {
        //         block_on(async {
        //             while let Some(m) = receiver.recv().await {
        //                 let data_writer = DataWriterAsync::new(
        //                     m.writer_address,
        //                     m.status_condition_address,
        //                     m.publisher,
        //                     m.topic,
        //                 );
        //                 match m.listener_operation {
        //                     PublisherListenerOperation::_LivelinessLost(status) => {
        //                         listener.on_liveliness_lost(data_writer, status).await
        //                     }
        //                     PublisherListenerOperation::OfferedDeadlineMissed(status) => {
        //                         listener
        //                             .on_offered_deadline_missed(data_writer, status)
        //                             .await
        //                     }
        //                     PublisherListenerOperation::OfferedIncompatibleQos(status) => {
        //                         listener
        //                             .on_offered_incompatible_qos(data_writer, status)
        //                             .await
        //                     }
        //                     PublisherListenerOperation::PublicationMatched(status) => {
        //                         listener.on_publication_matched(data_writer, status).await
        //                     }
        //                 }
        //             }
        //         });
        //     })
        //     .expect("failed to spawn thread");
        // Self { thread, sender }
    }

    fn sender(&self) -> &MpscSender<PublisherListenerMessage> {
        &self.sender
    }

    fn join(self) -> DdsResult<()> {
        self.sender.close();
        self.thread.join()?;
        Ok(())
    }
}
