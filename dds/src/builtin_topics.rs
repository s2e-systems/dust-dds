use crate::{
    implementation::{
        data_representation_builtin_endpoints::{
            discovered_topic_data::ReliabilityQosPolicyDataReaderAndTopicsDeserialize,
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
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
            DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER,
        },
    },
    topic_definition::type_support::{
        DdsDeserialize, DdsSerialize, DdsType, RepresentationType, PL_CDR_LE,
    },
};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct BuiltInTopicKey {
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct ParticipantBuiltinTopicData {
    key: BuiltInTopicKey,
    user_data: UserDataQosPolicy,
}

impl ParticipantBuiltinTopicData {
    pub fn new(key: BuiltInTopicKey, user_data: UserDataQosPolicy) -> Self {
        Self { key, user_data }
    }

    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }
}
impl DdsSerialize for ParticipantBuiltinTopicData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

    fn dds_serialize_parameter_list<W: std::io::Write>(
        &self,
        serializer: &mut crate::implementation::parameter_list_serde::parameter_list_serializer::ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        serializer.serialize_parameter(PID_PARTICIPANT_GUID, &self.key)?;
        serializer.serialize_parameter_if_not_default(PID_USER_DATA, &self.user_data)
    }
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
        let user_data = param_list.get_or_default(PID_USER_DATA)?;

        Ok(ParticipantBuiltinTopicData {
            key: participant_key,
            user_data,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct TopicBuiltinTopicData {
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
        self.name.as_ref()
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name.as_ref()
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
        let durability = param_list.get_or_default(PID_DURABILITY)?;
        let deadline = param_list.get_or_default(PID_DEADLINE)?;
        let latency_budget = param_list.get_or_default(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default(PID_LIVELINESS)?;
        let reliability = param_list
            .get_or_default::<ReliabilityQosPolicyDataReaderAndTopicsDeserialize>(PID_RELIABILITY)?
            .into();
        let transport_priority = param_list.get_or_default(PID_TRANSPORT_PRIORITY)?;
        let lifespan = param_list.get_or_default(PID_LIFESPAN)?;
        let ownership = param_list.get_or_default(PID_OWNERSHIP)?;
        let destination_order = param_list.get_or_default(PID_DESTINATION_ORDER)?;
        let history = param_list.get_or_default(PID_HISTORY)?;
        let resource_limits = param_list.get_or_default(PID_RESOURCE_LIMITS)?;
        let topic_data = param_list.get_or_default(PID_TOPIC_DATA)?;

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

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Into,
    derive_more::From,
)]
struct ReliabilityQosPolicyDataWriter(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataWriter {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)
    }
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    serde::Serialize,
    serde::Deserialize,
    derive_more::Into,
    derive_more::From,
)]
struct ReliabilityQosPolicyDataReader(ReliabilityQosPolicy);
impl Default for ReliabilityQosPolicyDataReader {
    fn default() -> Self {
        Self(DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PublicationBuiltinTopicData {
    key: BuiltInTopicKey,
    participant_key: BuiltInTopicKey,
    topic_name: String,
    type_name: String,
    durability: DurabilityQosPolicy,
    deadline: DeadlineQosPolicy,
    latency_budget: LatencyBudgetQosPolicy,
    liveliness: LivelinessQosPolicy,
    reliability: ReliabilityQosPolicyDataWriter,
    lifespan: LifespanQosPolicy,
    user_data: UserDataQosPolicy,
    ownership: OwnershipQosPolicy,
    destination_order: DestinationOrderQosPolicy,
    presentation: PresentationQosPolicy,
    partition: PartitionQosPolicy,
    topic_data: TopicDataQosPolicy,
    group_data: GroupDataQosPolicy,
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
            key,
            participant_key,
            topic_name,
            type_name,
            durability,
            deadline,
            latency_budget,
            liveliness,
            reliability: reliability.into(),
            lifespan,
            user_data,
            ownership,
            destination_order,
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
        self.topic_name.as_ref()
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name.as_ref()
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
        &self.reliability.0
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
        let participant_key = param_list.get_or_default(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default(PID_DURABILITY)?;
        let deadline = param_list.get_or_default(PID_DEADLINE)?;
        let latency_budget = param_list.get_or_default(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default(PID_LIVELINESS)?;
        let reliability = param_list.get_or_default(PID_RELIABILITY)?;
        let lifespan = param_list.get_or_default(PID_LIFESPAN)?;
        let user_data = param_list.get_or_default(PID_USER_DATA)?;
        let ownership = param_list.get_or_default(PID_OWNERSHIP)?;
        let destination_order = param_list.get_or_default(PID_DESTINATION_ORDER)?;
        let presentation = param_list.get_or_default(PID_PRESENTATION)?;
        let partition = param_list.get_or_default(PID_PARTITION)?;
        let topic_data = param_list.get_or_default(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default(PID_GROUP_DATA)?;

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

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
pub struct SubscriptionBuiltinTopicData {
    key: BuiltInTopicKey,
    participant_key: BuiltInTopicKey,
    topic_name: String,
    type_name: String,

    durability: DurabilityQosPolicy,
    deadline: DeadlineQosPolicy,
    latency_budget: LatencyBudgetQosPolicy,
    liveliness: LivelinessQosPolicy,
    reliability: ReliabilityQosPolicyDataReader,
    ownership: OwnershipQosPolicy,
    destination_order: DestinationOrderQosPolicy,
    user_data: UserDataQosPolicy,
    time_based_filter: TimeBasedFilterQosPolicy,

    presentation: PresentationQosPolicy,
    partition: PartitionQosPolicy,
    topic_data: TopicDataQosPolicy,
    group_data: GroupDataQosPolicy,
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
            reliability: reliability.into(),
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
        self.topic_name.as_ref()
    }

    pub fn get_type_name(&self) -> &str {
        self.type_name.as_ref()
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
        &self.reliability.0
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

impl DdsType for SubscriptionBuiltinTopicData {
    fn type_name() -> &'static str {
        "SubscriptionBuiltinTopicData"
    }
}
impl DdsSerialize for SubscriptionBuiltinTopicData {
    const REPRESENTATION_IDENTIFIER: RepresentationType = PL_CDR_LE;

    fn dds_serialize_parameter_list<W: std::io::Write>(
        &self,
        serializer: &mut crate::implementation::parameter_list_serde::parameter_list_serializer::ParameterListSerializer<W>,
    ) -> DdsResult<()> {
        serializer.serialize_parameter(PID_ENDPOINT_GUID, self.key())?;
        // Default value is a deviation from the standard and is used for interoperability reasons:
        serializer
            .serialize_parameter_if_not_default(PID_PARTICIPANT_GUID, &self.participant_key)?;
        serializer.serialize_parameter(PID_TOPIC_NAME, &self.topic_name)?;
        serializer.serialize_parameter(PID_TYPE_NAME, &self.type_name)?;
        serializer.serialize_parameter_if_not_default(PID_DURABILITY, &self.durability)?;
        serializer.serialize_parameter_if_not_default(PID_DEADLINE, &self.deadline)?;
        serializer.serialize_parameter_if_not_default(PID_LATENCY_BUDGET, &self.latency_budget)?;
        serializer.serialize_parameter_if_not_default(PID_DURABILITY, &self.liveliness)?;
        serializer.serialize_parameter_if_not_default(PID_RELIABILITY, &self.reliability)?;
        serializer.serialize_parameter_if_not_default(PID_OWNERSHIP, &self.ownership)?;
        serializer
            .serialize_parameter_if_not_default(PID_DESTINATION_ORDER, &self.destination_order)?;
        serializer.serialize_parameter_if_not_default(PID_USER_DATA, &self.user_data)?;
        serializer
            .serialize_parameter_if_not_default(PID_TIME_BASED_FILTER, &self.time_based_filter)?;
        serializer.serialize_parameter_if_not_default(PID_PRESENTATION, &self.presentation)?;
        serializer.serialize_parameter_if_not_default(PID_PARTITION, &self.partition)?;
        serializer.serialize_parameter_if_not_default(PID_TOPIC_DATA, &self.topic_data)?;
        serializer.serialize_parameter_if_not_default(PID_GROUP_DATA, &self.group_data)
    }
}
impl<'de> DdsDeserialize<'de> for SubscriptionBuiltinTopicData {
    fn deserialize(buf: &mut &'de [u8]) -> DdsResult<Self> {
        let param_list = ParameterListDeserializer::read(buf)?;

        // subscription_builtin_topic_data
        let key = param_list.get::<BuiltInTopicKey>(PID_ENDPOINT_GUID)?;
        // Default value is a deviation from the standard and is used for interoperability reasons
        let participant_key = param_list.get_or_default(PID_PARTICIPANT_GUID)?;
        let topic_name = param_list.get(PID_TOPIC_NAME)?;
        let type_name = param_list.get(PID_TYPE_NAME)?;
        let durability = param_list.get_or_default(PID_DURABILITY)?;
        let deadline = param_list.get_or_default(PID_DEADLINE)?;
        let latency_budget = param_list.get_or_default(PID_LATENCY_BUDGET)?;
        let liveliness = param_list.get_or_default(PID_LIVELINESS)?;
        let reliability = param_list.get_or_default(PID_RELIABILITY)?;
        let user_data = param_list.get_or_default(PID_USER_DATA)?;
        let ownership = param_list.get_or_default(PID_OWNERSHIP)?;
        let destination_order = param_list.get_or_default(PID_DESTINATION_ORDER)?;
        let time_based_filter = param_list.get_or_default(PID_TIME_BASED_FILTER)?;
        let presentation = param_list.get_or_default(PID_PRESENTATION)?;
        let partition = param_list.get_or_default(PID_PARTITION)?;
        let topic_data = param_list.get_or_default(PID_TOPIC_DATA)?;
        let group_data = param_list.get_or_default(PID_GROUP_DATA)?;

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
