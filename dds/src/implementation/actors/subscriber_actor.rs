use super::{
    any_data_reader_listener::AnyDataReaderListener, data_reader_actor::DataReaderActor,
    domain_participant_actor::DomainParticipantActor, handle::SubscriberHandle,
    topic_actor::TopicActor,
};
use crate::{
    dds_async::{
        subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync,
        topic::TopicAsync,
    },
    implementation::{
        actor::ActorAddress,
        actors::{
            domain_participant_actor, handle::DataReaderHandle,
            status_condition_actor::StatusConditionActor,
        },
        data_representation_builtin_endpoints::discovered_writer_data::DiscoveredWriterData,
        runtime::mpsc::MpscSender,
    },
    infrastructure::{
        error::DdsResult,
        instance::InstanceHandle,
        qos::{DataReaderQos, QosKind, SubscriberQos},
        qos_policy::PartitionQosPolicy,
        status::{
            LivelinessChangedStatus, RequestedDeadlineMissedStatus, RequestedIncompatibleQosStatus,
            SampleLostStatus, SampleRejectedStatus, StatusKind, SubscriptionMatchedStatus,
        },
    },
    rtps::{
        reader::{ReaderCacheChange, ReaderHistoryCache, TransportReader},
        transport::Transport,
        types::TopicKind,
    },
};
use fnmatch_regex::glob_to_regex;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::JoinHandle,
};
use tracing::warn;

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
    pub reader_address: ActorAddress<DataReaderActor>,
    pub status_condition_address: ActorAddress<StatusConditionActor>,
    pub subscriber: SubscriberAsync,
    pub topic: TopicAsync,
}

struct SubscriberListenerThread {
    thread: JoinHandle<()>,
    sender: MpscSender<SubscriberListenerMessage>,
}

impl SubscriberListenerThread {
    fn new(mut listener: Box<dyn SubscriberListenerAsync + Send>) -> Self {
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

pub struct SubscriberActor {
    subscriber_handle: SubscriberHandle,
    qos: SubscriberQos,
    data_reader_list: HashMap<InstanceHandle, DataReaderActor>,
    enabled: bool,
    data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: StatusConditionActor,
    subscriber_listener_thread: Option<SubscriberListenerThread>,
    subscriber_status_kind: Vec<StatusKind>,
}

impl SubscriberActor {
    pub fn new(
        qos: SubscriberQos,
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        subscriber_status_kind: Vec<StatusKind>,
        subscriber_handle: SubscriberHandle,
    ) -> Self {
        let status_condition = StatusConditionActor::default();
        let subscriber_listener_thread = listener.map(SubscriberListenerThread::new);

        SubscriberActor {
            subscriber_handle,
            qos,
            data_reader_list: HashMap::new(),
            enabled: false,
            data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition,
            subscriber_listener_thread,
            subscriber_status_kind,
        }
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
                        "Invalid partition regex name on subscriber qos {:?}. Error {:?}",
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

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn is_empty(&self) -> bool {
        self.data_reader_list.is_empty()
    }

    pub fn create_datareader(
        &mut self,
        a_topic: &TopicActor,
        qos: QosKind<DataReaderQos>,
        a_listener: Option<Box<dyn AnyDataReaderListener + Send>>,
        mask: Vec<StatusKind>,
        transport_reader: Box<dyn TransportReader>,
    ) -> DdsResult<InstanceHandle> {
        let qos = match qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let topic_name = a_topic.get_name().to_string();
        let type_name = a_topic.get_type_name().to_string();

        let type_support = a_topic.get_type_support();

        let data_reader_counter = self.data_reader_counter;
        self.data_reader_counter += 1;
        let data_reader_handle = DataReaderHandle::new(
            self.subscriber_handle,
            a_topic.get_handle(),
            data_reader_counter,
        );

        let listener = None;
        let data_reader_status_kind = mask.to_vec();

        let mut data_reader = DataReaderActor::new(
            data_reader_handle,
            topic_name,
            type_name,
            type_support.clone(),
            qos,
            listener,
            data_reader_status_kind,
            transport_reader,
        );

        if self.enabled && self.qos.entity_factory.autoenable_created_entities {
            data_reader.enable();
        }

        self.data_reader_list
            .insert(data_reader_handle.into(), data_reader);

        Ok(data_reader_handle.into())
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_qos(&self) -> &SubscriberQos {
        &self.qos
    }

    pub fn lookup_datareader_by_topic_name(
        &mut self,
        topic_name: &str,
    ) -> Option<&mut DataReaderActor> {
        self.data_reader_list
            .values_mut()
            .find(|dw| dw.get_topic_name() == topic_name)
    }

    pub fn add_matched_writer(&mut self, discovered_writer_data: &DiscoveredWriterData) {
        if self.is_partition_matched(discovered_writer_data.dds_publication_data.partition()) {
            if let Some(dr) = self.data_reader_list.values_mut().find(|dr| {
                dr.get_topic_name() == discovered_writer_data.dds_publication_data.topic_name()
            }) {
                dr.add_matched_writer(&discovered_writer_data, &self.qos);
            }
        }
    }

    pub fn get_mut_datareader(&mut self, handle: InstanceHandle) -> &mut DataReaderActor {
        self.data_reader_list.get_mut(&handle).expect("Must exist")
    }

    pub fn get_datareader(&self, handle: InstanceHandle) -> &DataReaderActor {
        self.data_reader_list.get(&handle).expect("Must exist")
    }

    pub fn delete_datareader(&mut self, handle: InstanceHandle) {
        todo!()
        // if let Some(removed_reader) = self.data_reader_list.remove(&message.handle) {
        //     Ok(removed_reader)
        // } else {
        //     Err(DdsError::PreconditionNotMet(
        //         "Data reader can only be deleted from its parent subscriber".to_string(),
        //     ))
        // }
    }

    pub fn set_default_datareader_qos(&mut self, qos: QosKind<DataReaderQos>) -> DdsResult<()> {
        match qos {
            QosKind::Default => self.default_data_reader_qos = DataReaderQos::default(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                self.default_data_reader_qos = q;
            }
        }
        Ok(())
    }

    pub fn get_default_datareader_qos(&self) -> &DataReaderQos {
        &self.default_data_reader_qos
    }

    pub fn set_qos(&mut self, qos: SubscriberQos) -> DdsResult<()> {
        if self.enabled {
            self.qos.check_immutability(&qos)?;
        }

        self.qos = qos;

        Ok(())
    }

    pub fn get_handle(&self) -> SubscriberHandle {
        self.subscriber_handle
    }

    pub fn remove_matched_writer(&mut self, discovered_writer_handle: InstanceHandle) {
        for data_reader in self.data_reader_list.values() {
            // let data_reader_address = data_reader.address();
            // let subscriber_mask_listener = (
            //     self.subscriber_listener_thread
            //         .as_ref()
            //         .map(|l| l.sender().clone()),
            //     self.subscriber_status_kind.clone(),
            // );
            // data_reader.send_actor_mail(data_reader_actor::RemoveMatchedWriter {
            //     discovered_writer_handle: message.discovered_writer_handle,
            //     // data_reader_address,
            //     // subscriber: SubscriberAsync::new(
            //     //     message.subscriber_address.clone(),
            //     //     self.status_condition.address(),
            //     //     message.participant.clone(),
            //     // ),
            //     // subscriber_mask_listener,
            //     // participant_mask_listener: message.participant_mask_listener.clone(),
            // });
        }
    }

    pub fn add_builtin_change(
        &mut self,
        cache_change: ReaderCacheChange,
        reader_instance_handle: InstanceHandle,
    ) {
        if let Some(reader) = self.data_reader_list.get_mut(&reader_instance_handle) {
            reader.add_change(cache_change);

            self.status_condition
                .add_communication_state(StatusKind::DataOnReaders);
        }
    }

    pub fn add_user_defined_change(&mut self, cache_change: ReaderCacheChange) {
        let writer_instance_handle = InstanceHandle::new(cache_change.writer_guid.into());
        for reader in self.data_reader_list.values_mut() {
            if reader
                .get_matched_publications()
                .contains(&writer_instance_handle)
            {
                reader.add_change(cache_change.clone());

                self.status_condition
                    .add_communication_state(StatusKind::DataOnReaders);
            }
        }
    }

    pub fn set_listener(
        &mut self,
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        status_kind: Vec<StatusKind>,
    ) -> DdsResult<()> {
        if let Some(l) = self.subscriber_listener_thread.take() {
            l.join()?;
        }
        self.subscriber_listener_thread = listener.map(SubscriberListenerThread::new);
        self.subscriber_status_kind = status_kind;

        Ok(())
    }
}
