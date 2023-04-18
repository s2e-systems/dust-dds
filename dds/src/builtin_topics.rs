use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_topic_data::ReliabilityQosPolicyDataReaderAndTopicsDeserialize,
            discovered_writer_data::ReliabilityQosPolicyDataWriterDeserialize,
            parameter_id_values::{
                PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID,
                PID_GROUP_DATA, PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS,
                PID_OWNERSHIP, PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION,
                PID_RELIABILITY, PID_RESOURCE_LIMITS, PID_TIME_BASED_FILTER, PID_TOPIC_DATA,
                PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY, PID_TYPE_NAME, PID_USER_DATA,
            },
        },
        parameter_list_serde::parameter_list_deserializer::ParameterListDeserializer,
    },
    infrastructure::{
        error::DdsResult,
        qos_policy::{
            DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
            HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
            OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
            ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
            TransportPriorityQosPolicy, UserDataQosPolicy,
        },
    },
    topic_definition::type_support::{DdsDeserialize, DdsType},
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct BuiltInTopicKey {
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ParticipantBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub user_data: UserDataQosPolicy,
}

impl DdsType for ParticipantBuiltinTopicData {
    fn type_name() -> &'static str {
        "ParticipantBuiltinTopicData"
    }
}

impl<'de> DdsDeserialize<'de> for ParticipantBuiltinTopicData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        let participant_key = param_list.get(PID_PARTICIPANT_GUID)?;
        let user_data = param_list.get_or_default::<UserDataQosPolicy, _>(PID_USER_DATA)?;

        Ok(ParticipantBuiltinTopicData {
            key: participant_key,
            user_data,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct TopicBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub name: String,
    pub type_name: String,
    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub transport_priority: TransportPriorityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub history: HistoryQosPolicy,
    pub resource_limits: ResourceLimitsQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub topic_data: TopicDataQosPolicy,
}

impl DdsType for TopicBuiltinTopicData {
    fn type_name() -> &'static str {
        "TopicBuiltinTopicData"
    }
}

impl<'de> DdsDeserialize<'de> for TopicBuiltinTopicData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        let key = param_list.get::<BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        let name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default::<DurabilityQosPolicy, _>(PID_DURABILITY)?;
        let deadline = param_list.get_or_default::<DeadlineQosPolicy, _>(PID_DEADLINE)?;
        let latency_budget =
            param_list.get_or_default::<LatencyBudgetQosPolicy, _>(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default::<LivelinessQosPolicy, _>(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize, _>(
                PID_RELIABILITY,
            )?;
        let transport_priority =
            param_list.get_or_default::<TransportPriorityQosPolicy, _>(PID_TRANSPORT_PRIORITY)?;
        let lifespan = param_list.get_or_default::<LifespanQosPolicy, _>(PID_LIFESPAN)?;
        let ownership = param_list.get_or_default::<OwnershipQosPolicy, _>(PID_OWNERSHIP)?;
        let destination_order =
            param_list.get_or_default::<DestinationOrderQosPolicy, _>(PID_DESTINATION_ORDER)?;
        let history = param_list.get_or_default::<HistoryQosPolicy, _>(PID_HISTORY)?;
        let resource_limits =
            param_list.get_or_default::<ResourceLimitsQosPolicy, _>(PID_RESOURCE_LIMITS)?;
        let topic_data = param_list.get_or_default::<TopicDataQosPolicy, _>(PID_TOPIC_DATA)?;

        Ok(Self {
            key,
            name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            transport_priority,
            lifespan,
            ownership,
            destination_order,
            history,
            resource_limits,
            topic_data,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicationBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub participant_key: BuiltInTopicKey,
    pub topic_name: String,
    pub type_name: String,

    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub lifespan: LifespanQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,

    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub topic_data: TopicDataQosPolicy,
    pub group_data: GroupDataQosPolicy,
}

impl DdsType for PublicationBuiltinTopicData {
    fn type_name() -> &'static str {
        "PublicationBuiltinTopicData"
    }
}

impl<'de> DdsDeserialize<'de> for PublicationBuiltinTopicData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        // publication_builtin_topic_data
        let key = param_list.get::<BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        // Default value is a deviation from the standard and is used for interoperability reasons
        let participant_key =
            param_list.get_or_default::<BuiltInTopicKey, _>(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default::<DurabilityQosPolicy, _>(PID_DURABILITY)?;
        let deadline = param_list.get_or_default::<DeadlineQosPolicy, _>(PID_DEADLINE)?;
        let latency_budget =
            param_list.get_or_default::<LatencyBudgetQosPolicy, _>(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default::<LivelinessQosPolicy, _>(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataWriterDeserialize, _>(PID_RELIABILITY)?;
        let lifespan = param_list.get_or_default::<LifespanQosPolicy, _>(PID_LIFESPAN)?;
        let user_data = param_list.get_or_default::<UserDataQosPolicy, _>(PID_USER_DATA)?;
        let ownership = param_list.get_or_default::<OwnershipQosPolicy, _>(PID_OWNERSHIP)?;
        let destination_order =
            param_list.get_or_default::<DestinationOrderQosPolicy, _>(PID_DESTINATION_ORDER)?;
        let presentation =
            param_list.get_or_default::<PresentationQosPolicy, _>(PID_PRESENTATION)?;
        let partition = param_list.get_or_default::<PartitionQosPolicy, _>(PID_PARTITION)?;
        let topic_data = param_list.get_or_default::<TopicDataQosPolicy, _>(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default::<GroupDataQosPolicy, _>(PID_GROUP_DATA)?;

        Ok(Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            lifespan,
            user_data,
            ownership,
            destination_order,
            presentation,
            partition,
            topic_data,
            group_data,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SubscriptionBuiltinTopicData {
    pub key: BuiltInTopicKey,
    pub participant_key: BuiltInTopicKey,
    pub topic_name: String,
    pub type_name: String,

    pub durability: DurabilityQosPolicy,
    pub deadline: DeadlineQosPolicy,
    pub latency_budget: LatencyBudgetQosPolicy,
    pub liveliness: LivelinessQosPolicy,
    pub reliability: ReliabilityQosPolicy,
    pub ownership: OwnershipQosPolicy,
    pub destination_order: DestinationOrderQosPolicy,
    pub user_data: UserDataQosPolicy,
    pub time_based_filter: TimeBasedFilterQosPolicy,

    pub presentation: PresentationQosPolicy,
    pub partition: PartitionQosPolicy,
    pub topic_data: TopicDataQosPolicy,
    pub group_data: GroupDataQosPolicy,
}

impl DdsType for SubscriptionBuiltinTopicData {
    fn type_name() -> &'static str {
        "SubscriptionBuiltinTopicData"
    }
}

impl<'de> DdsDeserialize<'de> for SubscriptionBuiltinTopicData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        // subscription_builtin_topic_data
        let key = param_list.get::<BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        // Default value is a deviation from the standard and is used for interoperability reasons
        let participant_key =
            param_list.get_or_default::<BuiltInTopicKey, _>(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default::<DurabilityQosPolicy, _>(PID_DURABILITY)?;
        let deadline = param_list.get_or_default::<DeadlineQosPolicy, _>(PID_DEADLINE)?;
        let latency_budget =
            param_list.get_or_default::<LatencyBudgetQosPolicy, _>(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default::<LivelinessQosPolicy, _>(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize, _>(
                PID_RELIABILITY,
            )?;
        let user_data = param_list.get_or_default::<UserDataQosPolicy, _>(PID_USER_DATA)?;
        let ownership = param_list.get_or_default::<OwnershipQosPolicy, _>(PID_OWNERSHIP)?;
        let destination_order =
            param_list.get_or_default::<DestinationOrderQosPolicy, _>(PID_DESTINATION_ORDER)?;
        let time_based_filter =
            param_list.get_or_default::<TimeBasedFilterQosPolicy, _>(PID_TIME_BASED_FILTER)?;
        let presentation =
            param_list.get_or_default::<PresentationQosPolicy, _>(PID_PRESENTATION)?;
        let partition = param_list.get_or_default::<PartitionQosPolicy, _>(PID_PARTITION)?;
        let topic_data = param_list.get_or_default::<TopicDataQosPolicy, _>(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default::<GroupDataQosPolicy, _>(PID_GROUP_DATA)?;

        Ok(Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            user_data,
            ownership,
            destination_order,
            time_based_filter,
            presentation,
            partition,
            topic_data,
            group_data,
        })
    }
}
