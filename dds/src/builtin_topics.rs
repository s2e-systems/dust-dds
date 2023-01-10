use crate::{
    implementation::{
        data_representation_builtin_endpoints::parameter_id_values::{
            PID_PARTICIPANT_GUID, PID_USER_DATA,
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

        let participant_key = param_list.get::<BuiltInTopicKey, _>(PID_PARTICIPANT_GUID)?;
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
