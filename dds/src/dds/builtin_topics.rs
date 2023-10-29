use crate::{
    cdr::{
        deserialize::CdrDeserialize,
        parameter_list_deserialize::ParameterListDeserialize,
        parameter_list_deserializer::ParameterListDeserializer,
        representation::{CdrRepresentation, CdrRepresentationKind},
        serialize::CdrSerialize,
    },
    implementation::data_representation_builtin_endpoints::parameter_id_values::{
        PID_DEADLINE, PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID, PID_GROUP_DATA,
        PID_HISTORY, PID_LATENCY_BUDGET, PID_LIFESPAN, PID_LIVELINESS, PID_OWNERSHIP,
        PID_PARTICIPANT_GUID, PID_PARTITION, PID_PRESENTATION, PID_RELIABILITY,
        PID_RESOURCE_LIMITS, PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME,
        PID_TRANSPORT_PRIORITY, PID_TYPE_NAME, PID_USER_DATA,
    },
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy, GroupDataQosPolicy,
        HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy, LivelinessQosPolicy,
        OwnershipQosPolicy, PartitionQosPolicy, PresentationQosPolicy, ReliabilityQosPolicy,
        ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy, TopicDataQosPolicy,
        TransportPriorityQosPolicy, UserDataQosPolicy,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
    },
    topic_definition::type_support::DdsHasKey,
};

#[derive(
    Debug, Clone, PartialEq, Eq, CdrSerialize, CdrDeserialize, derive_more::From, derive_more::AsRef,
)]
pub struct ReliabilityQosPolicyTopics(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyTopics {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize, Default)]
pub struct BuiltInTopicKey {
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

impl From<[u8; 16]> for BuiltInTopicKey {
    fn from(value: [u8; 16]) -> Self {
        BuiltInTopicKey { value }
    }
}

impl From<BuiltInTopicKey> for [u8; 16] {
    fn from(value: BuiltInTopicKey) -> Self {
        value.value
    }
}

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct ParticipantBuiltinTopicData {
    key: BuiltInTopicKey,
    user_data: UserDataQosPolicy,
}

impl<'de> ParameterListDeserialize<'de> for ParticipantBuiltinTopicData {
    fn deserialize(
        pl_deserializer: &mut ParameterListDeserializer<'de>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            key: pl_deserializer.read(PID_PARTICIPANT_GUID)?,
            user_data: pl_deserializer.read_with_default(PID_USER_DATA, Default::default())?,
        })
    }
}

impl ParticipantBuiltinTopicData {
    pub fn new(key: BuiltInTopicKey, user_data: UserDataQosPolicy) -> Self {
        Self {
            key: key,
            user_data: user_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }
}

impl DdsHasKey for ParticipantBuiltinTopicData {
    const HAS_KEY: bool = true;
}

impl CdrRepresentation for ParticipantBuiltinTopicData {
    const REPRESENTATION: CdrRepresentationKind = CdrRepresentationKind::PlCdrLe;
}

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct TopicBuiltinTopicData {
    key: BuiltInTopicKey,            //Parameter<PID_ENDPOINT_GUID, BuiltInTopicKey>,
    name: String,                    //Parameter<PID_TOPIC_NAME, String>,
    type_name: String,               //Parameter<PID_TYPE_NAME, String>,
    durability: DurabilityQosPolicy, //ParameterWithDefault<PID_DURABILITY, DurabilityQosPolicy>,
    deadline: DeadlineQosPolicy,     //ParameterWithDefault<PID_DEADLINE, DeadlineQosPolicy>,
    latency_budget: LatencyBudgetQosPolicy, //ParameterWithDefault<PID_LATENCY_BUDGET, LatencyBudgetQosPolicy>,
    liveliness: LivelinessQosPolicy, //ParameterWithDefault<PID_LIVELINESS, LivelinessQosPolicy>,
    reliability: ReliabilityQosPolicy, //ParameterWithDefault<PID_RELIABILITY, ReliabilityQosPolicyTopics>,
    transport_priority: TransportPriorityQosPolicy, //ParameterWithDefault<PID_TRANSPORT_PRIORITY, TransportPriorityQosPolicy>,
    lifespan: LifespanQosPolicy, //ParameterWithDefault<PID_LIFESPAN, LifespanQosPolicy>,
    destination_order: DestinationOrderQosPolicy, //ParameterWithDefault<PID_DESTINATION_ORDER, DestinationOrderQosPolicy>,
    history: HistoryQosPolicy, //ParameterWithDefault<PID_HISTORY, HistoryQosPolicy>,
    resource_limits: ResourceLimitsQosPolicy, //ParameterWithDefault<PID_RESOURCE_LIMITS, ResourceLimitsQosPolicy>,
    ownership: OwnershipQosPolicy, //ParameterWithDefault<PID_OWNERSHIP, OwnershipQosPolicy>,
    topic_data: TopicDataQosPolicy, // ParameterWithDefault<PID_TOPIC_DATA, TopicDataQosPolicy>,
}

impl TopicBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: BuiltInTopicKey,
        name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        transport_priority: TransportPriorityQosPolicy,
        lifespan: LifespanQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        history: HistoryQosPolicy,
        resource_limits: ResourceLimitsQosPolicy,
        ownership: OwnershipQosPolicy,
        topic_data: TopicDataQosPolicy,
    ) -> Self {
        Self {
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
            destination_order,
            history,
            resource_limits,
            ownership,
            topic_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    pub fn transport_priority(&self) -> &TransportPriorityQosPolicy {
        &self.transport_priority
    }

    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    pub fn history(&self) -> &HistoryQosPolicy {
        &self.history
    }

    pub fn resource_limits(&self) -> &ResourceLimitsQosPolicy {
        &self.resource_limits
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }
}

impl DdsHasKey for TopicBuiltinTopicData {
    const HAS_KEY: bool = true;
}

impl CdrRepresentation for TopicBuiltinTopicData {
    const REPRESENTATION: CdrRepresentationKind = CdrRepresentationKind::PlCdrLe;
}

#[derive(
    Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize, derive_more::From, derive_more::AsRef,
)]
struct ReliabilityQosPolicyDataWriter(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataWriter {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize, derive_more::From, derive_more::AsRef,
)]
struct ReliabilityQosPolicyDataReader(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataReader {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct PublicationBuiltinTopicData {
    key: BuiltInTopicKey, //Parameter<PID_ENDPOINT_GUID, BuiltInTopicKey>,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    participant_key: BuiltInTopicKey, //ParameterWithDefault<PID_PARTICIPANT_GUID, BuiltInTopicKey>,
    topic_name: String,               //Parameter<PID_TOPIC_NAME, String>,
    type_name: String,                //Parameter<PID_TYPE_NAME, String>,
    durability: DurabilityQosPolicy,  //ParameterWithDefault<PID_DURABILITY, DurabilityQosPolicy>,
    deadline: DeadlineQosPolicy,      //ParameterWithDefault<PID_DEADLINE, DeadlineQosPolicy>,
    latency_budget: LatencyBudgetQosPolicy, //ParameterWithDefault<PID_LATENCY_BUDGET, LatencyBudgetQosPolicy>,
    liveliness: LivelinessQosPolicy, //ParameterWithDefault<PID_LIVELINESS, LivelinessQosPolicy>,
    reliability: ReliabilityQosPolicyDataWriter, //ParameterWithDefault<PID_RELIABILITY, ReliabilityQosPolicyDataWriter>,
    lifespan: LifespanQosPolicy, //ParameterWithDefault<PID_LIFESPAN, LifespanQosPolicy>,
    user_data: UserDataQosPolicy, //ParameterWithDefault<PID_USER_DATA, UserDataQosPolicy>,
    ownership: OwnershipQosPolicy, //ParameterWithDefault<PID_OWNERSHIP, OwnershipQosPolicy>,
    destination_order: DestinationOrderQosPolicy, //ParameterWithDefault<PID_DESTINATION_ORDER, DestinationOrderQosPolicy>,
    presentation: PresentationQosPolicy, //ParameterWithDefault<PID_PRESENTATION, PresentationQosPolicy>,
    partition: PartitionQosPolicy,       //ParameterWithDefault<PID_PARTITION, PartitionQosPolicy>,
    topic_data: TopicDataQosPolicy,      //ParameterWithDefault<PID_TOPIC_DATA, TopicDataQosPolicy>,
    group_data: GroupDataQosPolicy,      //ParameterWithDefault<PID_GROUP_DATA, GroupDataQosPolicy>,
}

impl PublicationBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        lifespan: LifespanQosPolicy,
        user_data: UserDataQosPolicy,
        ownership: OwnershipQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        topic_data: TopicDataQosPolicy,
        group_data: GroupDataQosPolicy,
    ) -> Self {
        Self {
            key: key,
            participant_key: participant_key,
            topic_name: topic_name,
            type_name: type_name,
            durability: durability,
            deadline: deadline,
            latency_budget: latency_budget,
            liveliness: liveliness,
            reliability: ReliabilityQosPolicyDataWriter::from(reliability),
            lifespan: lifespan,
            user_data: user_data,
            ownership: ownership,
            destination_order: destination_order,
            presentation: presentation,
            partition: partition,
            topic_data: topic_data,
            group_data: group_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        self.reliability.as_ref()
    }

    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }
}

impl DdsHasKey for PublicationBuiltinTopicData {
    const HAS_KEY: bool = true;
}

impl CdrRepresentation for PublicationBuiltinTopicData {
    const REPRESENTATION: CdrRepresentationKind = CdrRepresentationKind::PlCdrLe;
}

#[derive(Debug, PartialEq, Eq, Clone, CdrSerialize, CdrDeserialize)]
pub struct SubscriptionBuiltinTopicData {
    key: BuiltInTopicKey, //Parameter<PID_ENDPOINT_GUID, BuiltInTopicKey>,
    // Default value is a deviation from the standard and is used for interoperability reasons:
    participant_key: BuiltInTopicKey, //ParameterWithDefault<PID_PARTICIPANT_GUID, BuiltInTopicKey>,
    topic_name: String,               //Parameter<PID_TOPIC_NAME, String>,
    type_name: String,                //Parameter<PID_TYPE_NAME, String>,

    durability: DurabilityQosPolicy, //ParameterWithDefault<PID_DURABILITY, DurabilityQosPolicy>,
    deadline: DeadlineQosPolicy,     //ParameterWithDefault<PID_DEADLINE, DeadlineQosPolicy>,
    latency_budget: LatencyBudgetQosPolicy, //ParameterWithDefault<PID_LATENCY_BUDGET, LatencyBudgetQosPolicy>,
    liveliness: LivelinessQosPolicy, //ParameterWithDefault<PID_LIVELINESS, LivelinessQosPolicy>,
    reliability: ReliabilityQosPolicy, //ParameterWithDefault<PID_RELIABILITY, ReliabilityQosPolicyDataReader>,
    ownership: OwnershipQosPolicy,     //ParameterWithDefault<PID_OWNERSHIP, OwnershipQosPolicy>,
    destination_order: DestinationOrderQosPolicy, //ParameterWithDefault<PID_DESTINATION_ORDER, DestinationOrderQosPolicy>,
    user_data: UserDataQosPolicy, //ParameterWithDefault<PID_USER_DATA, UserDataQosPolicy>,
    time_based_filter: TimeBasedFilterQosPolicy, //ParameterWithDefault<PID_TIME_BASED_FILTER, TimeBasedFilterQosPolicy>,

    presentation: PresentationQosPolicy, //ParameterWithDefault<PID_PRESENTATION, PresentationQosPolicy>,
    partition: PartitionQosPolicy,       //ParameterWithDefault<PID_PARTITION, PartitionQosPolicy>,
    topic_data: TopicDataQosPolicy, // ParameterWithDefault<PID_TOPIC_DATA, TopicDataQosPolicy>,
    group_data: GroupDataQosPolicy, //ParameterWithDefault<PID_GROUP_DATA, GroupDataQosPolicy>,
}

impl SubscriptionBuiltinTopicData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key: BuiltInTopicKey,
        participant_key: BuiltInTopicKey,
        topic_name: String,
        type_name: String,
        durability: DurabilityQosPolicy,
        deadline: DeadlineQosPolicy,
        latency_budget: LatencyBudgetQosPolicy,
        liveliness: LivelinessQosPolicy,
        reliability: ReliabilityQosPolicy,
        ownership: OwnershipQosPolicy,
        destination_order: DestinationOrderQosPolicy,
        user_data: UserDataQosPolicy,
        time_based_filter: TimeBasedFilterQosPolicy,
        presentation: PresentationQosPolicy,
        partition: PartitionQosPolicy,
        topic_data: TopicDataQosPolicy,
        group_data: GroupDataQosPolicy,
    ) -> Self {
        Self {
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability,
            ownership,
            destination_order,
            user_data,
            time_based_filter,
            presentation,
            partition,
            topic_data,
            group_data,
        }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    pub fn time_based_filter(&self) -> &TimeBasedFilterQosPolicy {
        &self.time_based_filter
    }

    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }
}

impl DdsHasKey for SubscriptionBuiltinTopicData {
    const HAS_KEY: bool = true;
}

impl CdrRepresentation for SubscriptionBuiltinTopicData {
    const REPRESENTATION: CdrRepresentationKind = CdrRepresentationKind::PlCdrLe;
}
