use super::{
    any_data_writer_listener::AnyDataWriterListener, data_writer_actor::DataWriterActor,
    status_condition_actor::StatusConditionActor, topic_actor::TopicActor,
};
use crate::{
    dds_async::{
        publisher::PublisherAsync, publisher_listener::PublisherListenerAsync, topic::TopicAsync,
    },
    implementation::{
        actor::{Actor, ActorAddress},
        runtime::{executor::ExecutorHandle, mpsc::MpscSender},
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
    },
    rtps::stateful_writer::WriterHistoryCache,
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

pub struct PublisherActor {
    pub qos: PublisherQos,
    pub instance_handle: InstanceHandle,
    pub data_writer_list: Vec<DataWriterActor>,
    pub enabled: bool,
    pub default_datawriter_qos: DataWriterQos,
    pub publisher_listener_thread: Option<PublisherListenerThread>,
    pub status_kind: Vec<StatusKind>,
    pub status_condition: Actor<StatusConditionActor>,
}

impl PublisherActor {
    pub fn create_datawriter(
        &mut self,
        a_topic: &TopicActor,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        mask: Vec<StatusKind>,
        instance_handle: InstanceHandle,
        transport_writer: Box<dyn WriterHistoryCache>,
        executor_handle: &ExecutorHandle,
    ) -> DdsResult<(InstanceHandle, ActorAddress<StatusConditionActor>)> {
        let qos = match qos {
            QosKind::Default => self.default_datawriter_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let topic_name = a_topic.topic_name.clone();

        let type_name = a_topic.type_name.clone();
        let mut data_writer = DataWriterActor::new(
            transport_writer,
            topic_name,
            type_name,
            a_listener,
            mask,
            qos,
            instance_handle,
            executor_handle,
        );
        let data_writer_handle = data_writer.get_instance_handle();
        let writer_status_condition_address = data_writer.get_statuscondition();
        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            data_writer.enable();
        }

        self.data_writer_list.push(data_writer);

        Ok((data_writer_handle, writer_status_condition_address))
    }
}
