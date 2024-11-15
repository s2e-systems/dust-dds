use super::any_data_writer_listener::{AnyDataWriterListener, DataWriterListenerOperation};
use crate::{
    builtin_topics::{BuiltInTopicKey, PublicationBuiltinTopicData, SubscriptionBuiltinTopicData},
    dds_async::{publisher::PublisherAsync, topic::TopicAsync},
    implementation::{
        actor::{Actor, ActorAddress},
        actors::status_condition_actor::StatusConditionActor,
        data_representation_builtin_endpoints::discovered_reader_data::DiscoveredReaderData,
        data_representation_inline_qos::{
            parameter_id_values::{PID_KEY_HASH, PID_STATUS_INFO},
            types::{
                STATUS_INFO_DISPOSED, STATUS_INFO_DISPOSED_UNREGISTERED, STATUS_INFO_UNREGISTERED,
            },
        },
        runtime::{
            executor::{block_on, ExecutorHandle, TaskHandle},
            mpsc::{mpsc_channel, MpscSender},
        },
        xtypes_glue::key_and_instance_handle::{
            get_instance_handle_from_serialized_foo, get_instance_handle_from_serialized_key,
            get_serialized_key_from_serialized_foo,
        },
    },
    infrastructure::{
        error::{DdsError, DdsResult},
        instance::{InstanceHandle, HANDLE_NIL},
        qos::{DataWriterQos, PublisherQos, TopicQos},
        qos_policy::{
            DurabilityQosPolicyKind, HistoryQosPolicyKind, Length, QosPolicyId,
            ReliabilityQosPolicyKind, DATA_REPRESENTATION_QOS_POLICY_ID, DEADLINE_QOS_POLICY_ID,
            DESTINATIONORDER_QOS_POLICY_ID, DURABILITY_QOS_POLICY_ID, INVALID_QOS_POLICY_ID,
            LATENCYBUDGET_QOS_POLICY_ID, LIVELINESS_QOS_POLICY_ID, OWNERSHIP_QOS_POLICY_ID,
            PRESENTATION_QOS_POLICY_ID, RELIABILITY_QOS_POLICY_ID, XCDR_DATA_REPRESENTATION,
        },
        status::{
            OfferedDeadlineMissedStatus, OfferedIncompatibleQosStatus, PublicationMatchedStatus,
            QosPolicyCount, StatusKind,
        },
        time::{Duration, DurationKind, Time},
    },
    rtps::{
        cache_change::RtpsCacheChange,
        messages::submessage_elements::{Parameter, ParameterList},
        stateful_writer::WriterHistoryCache,
        types::{ChangeKind, SequenceNumber},
    },
    xtypes::{
        dynamic_type::DynamicType, serialize::XTypesSerialize, xcdr_serializer::Xcdr1LeSerializer,
    },
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
    thread::JoinHandle,
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

pub struct DataWriterActor {
    pub instance_handle: InstanceHandle,
    pub transport_writer: Box<dyn WriterHistoryCache>,
    pub topic_name: String,
    pub type_name: String,
    pub matched_subscription_list: HashMap<InstanceHandle, SubscriptionBuiltinTopicData>,
    pub publication_matched_status: PublicationMatchedStatus,
    pub incompatible_subscription_list: HashSet<InstanceHandle>,
    pub offered_incompatible_qos_status: OfferedIncompatibleQosStatus,
    pub enabled: bool,
    pub status_condition: Actor<StatusConditionActor>,
    pub data_writer_listener_thread: Option<DataWriterListenerThread>,
    pub status_kind: Vec<StatusKind>,
    pub max_seq_num: Option<SequenceNumber>,
    pub last_change_sequence_number: SequenceNumber,
    pub qos: DataWriterQos,
    pub registered_instance_list: HashSet<InstanceHandle>,
    pub offered_deadline_missed_status: OfferedDeadlineMissedStatus,
    pub instance_deadline_missed_task: HashMap<InstanceHandle, TaskHandle>,
    pub instance_samples: HashMap<InstanceHandle, VecDeque<SequenceNumber>>,
}
