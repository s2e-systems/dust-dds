use super::{
    any_data_writer_listener::AnyDataWriterListener,
    data_writer_actor::DataWriterActor,
    handle::{DataWriterHandle, PublisherHandle},
    status_condition_actor::StatusConditionActor,
    topic_actor::TopicActor,
};
use crate::{
    dds_async::{
        publisher::PublisherAsync, publisher_listener::PublisherListenerAsync, topic::TopicAsync,
    },
    implementation::{
        actor::{Actor, ActorAddress},
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        runtime::{executor::ExecutorHandle, mpsc::MpscSender},
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataWriterQos, PublisherQos, QosKind},
        qos_policy::PartitionQosPolicy,
        status::{
            LivelinessLostStatus, OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus,
            PublicationMatchedStatus, StatusKind,
        },
    },
    rtps::stateful_writer::WriterHistoryCache,
};
use fnmatch_regex::glob_to_regex;
use std::{collections::HashMap, thread::JoinHandle};
use tracing::warn;

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

struct PublisherListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<PublisherListenerMessage>,
}

impl PublisherListenerThread {
    fn new(mut listener: Box<dyn PublisherListenerAsync + Send>) -> Self {
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
    qos: PublisherQos,
    publisher_handle: PublisherHandle,
    data_writer_list: Vec<DataWriterActor>,
    enabled: bool,
    data_writer_counter: u8,
    default_datawriter_qos: DataWriterQos,
    publisher_listener_thread: Option<PublisherListenerThread>,
    status_kind: Vec<StatusKind>,
    status_condition: Actor<StatusConditionActor>,
}

impl PublisherActor {
    pub fn new(
        qos: PublisherQos,
        listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
        publisher_handle: PublisherHandle,
        executor_handle: &ExecutorHandle,
    ) -> Self {
        let publisher_listener_thread = listener.map(PublisherListenerThread::new);
        let status_condition = Actor::spawn(StatusConditionActor::default(), executor_handle);
        Self {
            qos,
            publisher_handle,
            data_writer_list: Vec::new(),
            enabled: false,
            data_writer_counter: 0,
            default_datawriter_qos: DataWriterQos::default(),
            publisher_listener_thread,
            status_kind,
            status_condition,
        }
    }

    pub fn get_statuscondition(&self) -> ActorAddress<StatusConditionActor> {
        self.status_condition.address()
    }

    fn is_partition_matched(&self, discovered_partition_qos_policy: &PartitionQosPolicy) -> bool {
        let is_any_name_matched = discovered_partition_qos_policy
            .name
            .iter()
            .any(|n| self.qos.partition.name.contains(n));

        let is_any_received_regex_matched_with_partition_qos = discovered_partition_qos_policy
            .name
            .iter()
            .filter_map(|n| match glob_to_regex(n) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    warn!(
                        "Received invalid partition regex name {:?}. Error {:?}",
                        n, e
                    );
                    None
                }
            })
            .any(|regex| self.qos.partition.name.iter().any(|n| regex.is_match(n)));

        let is_any_local_regex_matched_with_received_partition_qos = self
            .qos
            .partition
            .name
            .iter()
            .filter_map(|n| match glob_to_regex(n) {
                Ok(regex) => Some(regex),
                Err(e) => {
                    warn!(
                        "Invalid partition regex name on publisher qos {:?}. Error {:?}",
                        n, e
                    );
                    None
                }
            })
            .any(|regex| {
                discovered_partition_qos_policy
                    .name
                    .iter()
                    .any(|n| regex.is_match(n))
            });

        discovered_partition_qos_policy == &self.qos.partition
            || is_any_name_matched
            || is_any_received_regex_matched_with_partition_qos
            || is_any_local_regex_matched_with_received_partition_qos
    }

    pub fn create_datawriter(
        &mut self,
        a_topic: &TopicActor,
        qos: QosKind<DataWriterQos>,
        a_listener: Option<Box<dyn AnyDataWriterListener + Send>>,
        mask: Vec<StatusKind>,
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

        let topic_name = a_topic.get_name().to_string();

        let type_name = a_topic.get_type_name().to_string();
        let mut data_writer = DataWriterActor::new(
            transport_writer,
            topic_name,
            type_name,
            a_listener,
            mask,
            qos,
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

    pub fn delete_datawriter(&mut self, handle: &InstanceHandle) -> Option<DataWriterActor> {
        let data_writer_index = self
            .data_writer_list
            .iter()
            .position(|x| &x.get_instance_handle() == handle)?;
        Some(self.data_writer_list.remove(data_writer_index))
    }

    pub fn get_datawriter(&self, handle: &InstanceHandle) -> Option<&DataWriterActor> {
        self.data_writer_list
            .iter()
            .find(|x| &x.get_instance_handle() == handle)
    }

    pub fn get_mut_datawriter(&mut self, handle: &InstanceHandle) -> Option<&mut DataWriterActor> {
        self.data_writer_list
            .iter_mut()
            .find(|x| &x.get_instance_handle() == handle)
    }

    pub fn lookup_datawriter_by_topic_name(
        &mut self,
        topic_name: &str,
    ) -> Option<&mut DataWriterActor> {
        self.data_writer_list
            .iter_mut()
            .find(|dw| dw.get_topic_name() == topic_name)
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn get_qos(&self) -> &PublisherQos {
        &self.qos
    }

    pub fn add_matched_reader(&mut self, discovered_reader_data: &DiscoveredReaderData) {
        if self.is_partition_matched(
            discovered_reader_data
                .subscription_builtin_topic_data()
                .partition(),
        ) {
            for dw in self.data_writer_list.iter_mut().filter(|dw| {
                dw.get_topic_name()
                    == discovered_reader_data
                        .subscription_builtin_topic_data()
                        .topic_name()
            }) {
                dw.add_matched_reader(discovered_reader_data, &self.qos);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        self.data_writer_list.is_empty()
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_default_data_writer_qos(&mut self, qos: QosKind<DataWriterQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => DataWriterQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };
        self.default_datawriter_qos = qos;
        Ok(())
    }

    pub fn set_qos(&mut self, qos: QosKind<PublisherQos>) -> DdsResult<()> {
        let qos = match qos {
            QosKind::Default => Default::default(),
            QosKind::Specific(q) => q,
        };

        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }

    pub fn get_default_datawriter_qos(&self) -> &DataWriterQos {
        &self.default_datawriter_qos
    }

    pub fn get_instance_handle(&self) -> InstanceHandle {
        self.publisher_handle.into()
    }

    pub fn get_status_kind(&self) -> Vec<StatusKind> {
        self.status_kind.clone()
    }

    pub fn remove_matched_reader(&mut self, discovered_reader_handle: InstanceHandle) {
        for data_writer in self.data_writer_list.iter_mut() {
            data_writer.remove_matched_reader(discovered_reader_handle);
        }
    }

    pub fn delete_contained_entities(&mut self) -> Vec<DataWriterActor> {
        self.data_writer_list.drain(..).collect()
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Box<dyn PublisherListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        if let Some(l) = self.publisher_listener_thread.take() {
            l.join()?;
        }
        self.publisher_listener_thread = listener.map(PublisherListenerThread::new);
        self.status_kind = status_kind;
        Ok(())
    }
}

impl PublisherQos {
    fn check_immutability(&self, other: &Self) -> DdsResult<()> {
        if self.presentation != other.presentation {
            Err(DdsError::ImmutablePolicy)
        } else {
            Ok(())
        }
    }
}
