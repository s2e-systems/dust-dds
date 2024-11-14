use tracing::debug;

use super::{
    any_data_reader_listener::{AnyDataReaderListener, DataReaderListenerOperation},
    handle::DataReaderHandle,
    status_condition_actor::{self, StatusConditionActor},
};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    dds_async::{subscriber::SubscriberAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress},
        data_representation_builtin_endpoints::{
            discovered_reader_data::DiscoveredReaderData,
            discovered_writer_data::DiscoveredWriterData,
        },
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                StatusInfo, STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED,
                STATUS_INFO_UNREGISTERED,
            },
        },
        runtime::{
            executor::{block_on, ExecutorHandle, TaskHandle},
            mpsc::{mpsc_channel, MpscSender},
        },
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::InstanceHandle,
        qos::{DataReaderQos, SubscriberQos, TopicQos},
        qos_policy::{
            DestinationOrderQosPolicyKind, DurabilityQosPolicyKind, HistoryQosPolicyKind,
            OwnershipQosPolicyKind, QosPolicyId, ReliabilityQosPolicyKind,
            DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, LATENCYBUDGET_QOS_POLICY_ID,
            LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID, PRESENTATION_QOS_POLICY_ID,
            RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            LivelinessChangedStatus, QosPolicyCount, RequestedDeadlineMissedStatus,
            RequestedIncompatibleQosStatus, SampleLostStatus, SampleRejectedStatus,
            SampleRejectedStatusKind, StatusKind, SubscriptionMatchedStatus,
        },
        time::{DurationKind, Time},
    },
    rtps::{
        messages::submessage_elements::{Data, ParameterList},
        reader::{ReaderCacheChange, TransportReader},
        types::{ChangeKind, DurabilityKind, Guid, ReliabilityKind},
    },
    subscription::sample_info::{InstanceStateKind, SampleInfo, SampleStateKind, ViewStateKind},
    xtypes::{
        deserialize::XTypesDeserialize, dynamic_type::DynamicType,
        xcdr_deserializer::Xcdr1LeDeserializer,
    },
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread::JoinHandle,
};

pub struct InstanceState {
    pub view_state: ViewStateKind,
    pub instance_state: InstanceStateKind,
    pub most_recent_disposed_generation_count: i32,
    pub most_recent_no_writers_generation_count: i32,
}

impl InstanceState {
    pub fn new() -> Self {
        Self {
            view_state: ViewStateKind::New,
            instance_state: InstanceStateKind::Alive,
            most_recent_disposed_generation_count: 0,
            most_recent_no_writers_generation_count: 0,
        }
    }

    pub fn update_state(&mut self, change_kind: ChangeKind) {
        match self.instance_state {
            InstanceStateKind::Alive => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveDisposedUnregistered
                {
                    self.instance_state = InstanceStateKind::NotAliveDisposed;
                } else if change_kind == ChangeKind::NotAliveUnregistered {
                    self.instance_state = InstanceStateKind::NotAliveNoWriters;
                }
            }
            InstanceStateKind::NotAliveDisposed => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_disposed_generation_count += 1;
                }
            }
            InstanceStateKind::NotAliveNoWriters => {
                if change_kind == ChangeKind::Alive {
                    self.instance_state = InstanceStateKind::Alive;
                    self.most_recent_no_writers_generation_count += 1;
                }
            }
        }

        match self.view_state {
            ViewStateKind::New => (),
            ViewStateKind::NotNew => {
                if change_kind == ChangeKind::NotAliveDisposed
                    || change_kind == ChangeKind::NotAliveUnregistered
                {
                    self.view_state = ViewStateKind::New;
                }
            }
        }
    }

    pub fn mark_viewed(&mut self) {
        self.view_state = ViewStateKind::NotNew;
    }
}

#[derive(Debug)]
pub struct ReaderSample {
    pub kind: ChangeKind,
    pub writer_guid: Guid,
    pub instance_handle: InstanceHandle,
    pub source_timestamp: Option<Time>,
    pub data_value: Data,
    pub _inline_qos: ParameterList,
    pub sample_state: SampleStateKind,
    pub disposed_generation_count: i32,
    pub no_writers_generation_count: i32,
    pub reception_timestamp: Time,
}

pub struct IndexedSample {
    pub index: usize,
    pub sample: (Option<Data>, SampleInfo),
}

pub struct DataReaderListenerMessage {
    listener_operation: DataReaderListenerOperation,
    reader_address: ActorAddress<DataReaderActor>,
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
                                m.reader_address,
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

pub struct DataReaderActor {
    pub instance_handle: InstanceHandle,
    pub sample_list: Vec<ReaderSample>,
    pub qos: DataReaderQos,
    pub topic_name: String,
    pub type_name: String,
    pub type_support: Arc<dyn DynamicType + Send + Sync>,
    pub _liveliness_changed_status: LivelinessChangedStatus,
    pub requested_deadline_missed_status: RequestedDeadlineMissedStatus,
    pub requested_incompatible_qos_status: RequestedIncompatibleQosStatus,
    pub sample_lost_status: SampleLostStatus,
    pub sample_rejected_status: SampleRejectedStatus,
    pub subscription_matched_status: SubscriptionMatchedStatus,
    pub matched_publication_list: HashMap<InstanceHandle, PublicationBuiltinTopicData>,
    pub enabled: bool,
    pub data_available_status_changed_flag: bool,
    pub incompatible_writer_list: HashSet<InstanceHandle>,
    pub status_condition: Actor<StatusConditionActor>,
    pub data_reader_listener_thread: Option<DataReaderListenerThread>,
    pub data_reader_status_kind: Vec<StatusKind>,
    pub instances: HashMap<InstanceHandle, InstanceState>,
    pub instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
    pub instance_ownership: HashMap<InstanceHandle, Guid>,
    pub transport_reader: Box<dyn TransportReader>,
}
