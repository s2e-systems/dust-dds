use super::{
    any_data_reader_listener::AnyDataReaderListener, data_reader_actor::DataReaderActor,
    domain_participant_actor::DomainParticipantActor, topic_actor::TopicActor,
};
use crate::{
    dds_async::{
        subscriber::SubscriberAsync, subscriber_listener::SubscriberListenerAsync,
        topic::TopicAsync,
    },
    implementation::{
        actor::ActorAddress,
        actors::{domain_participant_actor, status_condition_actor::StatusConditionActor},
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
        group::RtpsGroup,
        participant::RtpsParticipant,
        reader::{ReaderCacheChange, ReaderHistoryCache},
        types::{EntityId, Guid, USER_DEFINED_READER_NO_KEY, USER_DEFINED_READER_WITH_KEY},
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
    qos: SubscriberQos,
    rtps_group: RtpsGroup,
    transport: Arc<Mutex<RtpsParticipant>>,
    data_reader_list: HashMap<InstanceHandle, DataReaderActor>,
    enabled: bool,
    user_defined_data_reader_counter: u8,
    default_data_reader_qos: DataReaderQos,
    status_condition: StatusConditionActor,
    subscriber_listener_thread: Option<SubscriberListenerThread>,
    subscriber_status_kind: Vec<StatusKind>,
}

impl SubscriberActor {
    pub fn new(
        qos: SubscriberQos,
        rtps_group: RtpsGroup,
        transport: Arc<Mutex<RtpsParticipant>>,
        listener: Option<Box<dyn SubscriberListenerAsync + Send>>,
        subscriber_status_kind: Vec<StatusKind>,
        data_reader_list: Vec<DataReaderActor>,
    ) -> Self {
        let status_condition = StatusConditionActor::default();
        let subscriber_listener_thread = listener.map(SubscriberListenerThread::new);
        let data_reader_list = data_reader_list
            .into_iter()
            .map(|dr| (dr.get_instance_handle(), dr))
            .collect();

        SubscriberActor {
            qos,
            rtps_group,
            transport,
            data_reader_list,
            enabled: false,
            user_defined_data_reader_counter: 0,
            default_data_reader_qos: Default::default(),
            status_condition,
            subscriber_listener_thread,
            subscriber_status_kind,
        }
    }

    fn get_unique_reader_id(&mut self) -> u8 {
        let counter = self.user_defined_data_reader_counter;
        self.user_defined_data_reader_counter += 1;
        counter
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
        domain_participant_address: ActorAddress<DomainParticipantActor>,
    ) -> DdsResult<Guid> {
        struct UserDefinedReaderHistoryCache {
            pub domain_participant_address: ActorAddress<DomainParticipantActor>,
            pub subscriber_guid: Guid,
            pub data_reader_guid: Guid,
        }

        impl ReaderHistoryCache for UserDefinedReaderHistoryCache {
            fn add_change(&mut self, cache_change: ReaderCacheChange) {
                self.domain_participant_address
                    .send_actor_mail(domain_participant_actor::AddCacheChange {
                        cache_change,
                        subscriber_guid: self.subscriber_guid,
                        data_reader_guid: self.data_reader_guid,
                    })
                    .ok();
            }
        }

        let qos = match qos {
            QosKind::Default => self.default_data_reader_qos.clone(),
            QosKind::Specific(q) => {
                q.is_consistent()?;
                q
            }
        };

        let guid_prefix = self.rtps_group.guid().prefix();
        let topic_name = a_topic.get_name().to_string();
        let type_name = a_topic.get_type_name().to_string();
        let type_support = a_topic.get_type_support();
        let has_key = {
            let mut has_key = false;
            for index in 0..type_support.get_member_count() {
                if type_support
                    .get_member_by_index(index)?
                    .get_descriptor()?
                    .is_key
                {
                    has_key = true;
                    break;
                }
            }
            has_key
        };

        let subscriber_guid = self.rtps_group.guid();
        let entity_kind = match has_key {
            true => USER_DEFINED_READER_WITH_KEY,
            false => USER_DEFINED_READER_NO_KEY,
        };
        let entity_key: [u8; 3] = [
            subscriber_guid.entity_id().entity_key()[0],
            self.get_unique_reader_id(),
            0,
        ];

        let entity_id = EntityId::new(entity_key, entity_kind);
        let data_reader_guid = Guid::new(subscriber_guid.prefix(), entity_id);

        let user_defined_reader_history_cache = UserDefinedReaderHistoryCache {
            domain_participant_address,
            subscriber_guid,
            data_reader_guid,
        };

        let rtps_reader = self.transport.lock().unwrap().create_reader(
            data_reader_guid,
            Box::new(user_defined_reader_history_cache),
        );

        let listener = None;
        let data_reader_status_kind = mask.to_vec();
        let data_reader = DataReaderActor::new(
            data_reader_guid,
            rtps_reader,
            topic_name,
            type_name,
            type_support.clone(),
            qos,
            listener,
            data_reader_status_kind,
        );

        self.data_reader_list
            .insert(InstanceHandle::new(data_reader_guid.into()), data_reader);

        Ok(data_reader_guid)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn get_qos(&self) -> &SubscriberQos {
        &self.qos
    }

    pub fn add_matched_writer(&mut self, discovered_writer_data: DiscoveredWriterData) {
        if self.is_partition_matched(discovered_writer_data.dds_publication_data.partition()) {
            if let Some(dr) = self.data_reader_list.values_mut().find(|dr| {
                dr.get_topic_name() == discovered_writer_data.dds_publication_data.topic_name()
            }) {
                dr.add_matched_writer(discovered_writer_data, &self.qos);
            }
        }
    }

    pub fn get_mut_datareader_by_guid(&mut self, datareader_guid: Guid) -> &mut DataReaderActor {
        self.data_reader_list
            .get_mut(&InstanceHandle::new(datareader_guid.into()))
            .expect("Must exist")
    }

    pub fn get_datareader_by_guid(&self, datareader_guid: Guid) -> &DataReaderActor {
        self.data_reader_list
            .get(&InstanceHandle::new(datareader_guid.into()))
            .expect("Must exist")
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

    pub fn get_guid(&self) -> Guid {
        self.rtps_group.guid()
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

    pub fn get_instance_handle(&self) -> InstanceHandle {
        InstanceHandle::new(self.rtps_group.guid().into())
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

    pub fn add_change(
        &mut self,
        cache_change: ReaderCacheChange,
        reader_instance_handle: InstanceHandle,
    ) {
        // if let Some(reader) = self.data_reader_list.get(&message.reader_instance_handle) {
        //     reader.send_actor_mail(data_reader_actor::AddChange {
        //         cache_change: message.cache_change,
        //     });
        // }

        self.status_condition
            .add_communication_state(StatusKind::DataOnReaders);
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
