use super::infrastructure::qos_policy::{
    DataRepresentationQosPolicy, DeadlineQosPolicy, DestinationOrderQosPolicy, DurabilityQosPolicy,
    GroupDataQosPolicy, HistoryQosPolicy, LatencyBudgetQosPolicy, LifespanQosPolicy,
    LivelinessQosPolicy, OwnershipQosPolicy, OwnershipStrengthQosPolicy, PartitionQosPolicy,
    PresentationQosPolicy, PropertyQosPolicy, ReliabilityQosPolicy, ResourceLimitsQosPolicy,
    TimeBasedFilterQosPolicy, TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
};
use crate::{
    dcps::data_representation_builtin_endpoints::parameter_id_values::{
        PID_AVAILABLE_BUILTIN_ENDPOINTS_EXT, PID_DATA_REPRESENTATION, PID_DEADLINE,
        PID_DESTINATION_ORDER, PID_DURABILITY, PID_ENDPOINT_GUID,
        PID_ENDPOINT_SECURITY_PROTECTION_INFO,
        PID_ENDPOINT_SECURITY_SYMMETRIC_CIPHER_ALGORITHM_INFO, PID_GROUP_DATA, PID_HISTORY,
        PID_IDENTITY_STATUS_TOKEN, PID_IDENTITY_TOKEN, PID_LATENCY_BUDGET, PID_LIFESPAN,
        PID_LIVELINESS, PID_OWNERSHIP, PID_OWNERSHIP_STRENGTH, PID_PARTICIPANT_GUID,
        PID_PARTICIPANT_SECURITY_BUILTIN_EP_SYMMETRIC_CIPHER_ALGORITHM_INFO,
        PID_PARTICIPANT_SECURITY_DIGITAL_SIGNATURE_ALGORITHM_INFO,
        PID_PARTICIPANT_SECURITY_KEY_ESTABLISHMENT_ALGORITHM_INFO,
        PID_PARTICIPANT_SECURITY_PROTECTION_INFO, PID_PARTITION, PID_PERMISSIONS_TOKEN,
        PID_PRESENTATION, PID_PROPERTY_LIST, PID_RELIABILITY, PID_RESOURCE_LIMITS,
        PID_TIME_BASED_FILTER, PID_TOPIC_DATA, PID_TOPIC_NAME, PID_TRANSPORT_PRIORITY,
        PID_TYPE_CONSISTENCY_ENFORCEMENT, PID_TYPE_INFORMATION, PID_TYPE_NAME, PID_USER_DATA,
    },
    infrastructure::qos_policy::{
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS,
        DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER, TypeConsistencyEnforcementQosPolicy,
    },
    security::types::{
        BuiltinEndpointSetExt, EndpointSecurityProtectionInfo,
        EndpointSecuritySymmetricCipherAlgorithmInfo, IdentityStatusToken, IdentityToken,
        ParticipantSecurityDigitalSignatureAlgorithmInfo,
        ParticipantSecurityKeyEstablishmentAlgorithmInfo, ParticipantSecurityProtectionInfo,
        ParticipantSecuritySymmetricCipherAlgorithmInfo, PermissionsToken,
    },
    xtypes::{
        type_object::TypeInformation,
        type_support::{_String, TypeSupport},
    },
};

/// Topic name of the built-in publication discovery topic
pub const DCPS_PUBLICATION: &str = "DCPSPublication";

/// Topic name of the built-in subscription discovery topic
pub const DCPS_SUBSCRIPTION: &str = "DCPSSubscription";

/// Topic name of the built-in topic discovery topic
pub const DCPS_TOPIC: &str = "DCPSTopic";

/// Topic name of the built-in participant discovery topic
pub const DCPS_PARTICIPANT: &str = "DCPSParticipant";

/// Topic name of the secure built-in participant discovery topic
pub const DCPS_PARTICIPANTS_SECURE: &str = "DCPSParticipantsSecure";

/// Structure representing the instance handle (or key) of an entity.
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final", nested)]
pub struct BuiltInTopicKey {
    /// InstanceHandle value as an array of 16 octets.
    pub value: [u8; 16], // Originally in the DDS idl [i32;3]
}

/// Structure representing a discovered [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant).
#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable")]
pub struct ParticipantBuiltinTopicData {
    #[dust_dds(id=PID_PARTICIPANT_GUID as u32, key)]
    pub(crate) key: BuiltInTopicKey,
    #[dust_dds(id=PID_USER_DATA as u32)]
    pub(crate) user_data: UserDataQosPolicy,
    #[dust_dds(id=PID_IDENTITY_TOKEN as u32, optional)]
    pub(crate) identity_token: Option<IdentityToken>,
    #[dust_dds(id=PID_PERMISSIONS_TOKEN as u32, optional)]
    pub(crate) permissions_token: Option<PermissionsToken>,
    #[dust_dds(id=PID_PARTICIPANT_SECURITY_PROTECTION_INFO as u32, optional)]
    pub(crate) protection_info: Option<ParticipantSecurityProtectionInfo>,
    #[dust_dds(id=PID_AVAILABLE_BUILTIN_ENDPOINTS_EXT as u32, optional)]
    pub(crate) available_builtin_endpoints_ext: Option<BuiltinEndpointSetExt>,
    #[dust_dds(id=PID_PARTICIPANT_SECURITY_DIGITAL_SIGNATURE_ALGORITHM_INFO as u32, optional)]
    pub(crate) digital_signature: Option<ParticipantSecurityDigitalSignatureAlgorithmInfo>,
    #[dust_dds(id=PID_PARTICIPANT_SECURITY_KEY_ESTABLISHMENT_ALGORITHM_INFO as u32, optional)]
    pub(crate) key_establishment: Option<ParticipantSecurityKeyEstablishmentAlgorithmInfo>,
    #[dust_dds(id=PID_PARTICIPANT_SECURITY_BUILTIN_EP_SYMMETRIC_CIPHER_ALGORITHM_INFO as u32, optional)]
    pub(crate) symmetric_cipher: Option<ParticipantSecuritySymmetricCipherAlgorithmInfo>,
    #[dust_dds(id=PID_PROPERTY_LIST as u32, optional)]
    pub(crate) property: Option<PropertyQosPolicy>,
}

impl ParticipantBuiltinTopicData {
    /// Get the key value of the discovered participant.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the user data value of the discovered participant.
    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    /// Get the identity token value of the discovered participant.
    pub fn identity_token(&self) -> &Option<IdentityToken> {
        &self.identity_token
    }

    /// Get the permissions token value of the discovered participant.
    pub fn permissions_token(&self) -> &Option<PermissionsToken> {
        &self.permissions_token
    }

    /// Get the protection info value of the discovered participant.
    pub fn protection_info(&self) -> &Option<ParticipantSecurityProtectionInfo> {
        &self.protection_info
    }

    /// Get the available builtin endpoints ext value of the discovered participant.
    pub fn available_builtin_endpoints_ext(&self) -> &Option<BuiltinEndpointSetExt> {
        &self.available_builtin_endpoints_ext
    }

    /// Get the digital signature value of the discovered participant.
    pub fn digital_signature(&self) -> &Option<ParticipantSecurityDigitalSignatureAlgorithmInfo> {
        &self.digital_signature
    }

    /// Get the key establishment value of the discovered participant.
    pub fn key_establishment(&self) -> &Option<ParticipantSecurityKeyEstablishmentAlgorithmInfo> {
        &self.key_establishment
    }

    /// Get the symmetric cipher value of the discovered participant.
    pub fn symmetric_cipher(&self) -> &Option<ParticipantSecuritySymmetricCipherAlgorithmInfo> {
        &self.symmetric_cipher
    }

    /// Get the property QoS policy value of the discovered participant.
    pub fn property(&self) -> &Option<PropertyQosPolicy> {
        &self.property
    }
}

/// Structure representing a discovered [`DomainParticipant`](crate::domain::domain_participant::DomainParticipant) secure data.
#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable", base_type = ParticipantBuiltinTopicData)]
pub struct ParticipantBuiltinTopicDataSecure {
    pub(crate) parent: ParticipantBuiltinTopicData,
    #[dust_dds(id=PID_IDENTITY_STATUS_TOKEN as u32, optional)]
    pub(crate) identity_status_token: Option<IdentityStatusToken>,
}

impl ParticipantBuiltinTopicDataSecure {
    /// Get the value of the base type.
    pub fn base_type(&self) -> &ParticipantBuiltinTopicData {
        &self.parent
    }

    /// Get the identity status token value of the discovered participant.
    pub fn identity_status_token(&self) -> Option<&IdentityStatusToken> {
        self.identity_status_token.as_ref()
    }
}

/// Structure representing a discovered [`Topic`](crate::topic_definition::topic::Topic).
#[derive(Debug, PartialEq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable")]
pub struct TopicBuiltinTopicData {
    #[dust_dds(id=PID_ENDPOINT_GUID as u32, key)]
    pub(crate) key: BuiltInTopicKey,
    #[dust_dds(id=PID_TOPIC_NAME as u32)]
    pub(crate) name: _String,
    #[dust_dds(id=PID_TYPE_NAME as u32)]
    pub(crate) type_name: _String,
    #[dust_dds(id=PID_TYPE_INFORMATION as u32, optional)]
    pub(crate) type_information: Option<TypeInformation>,
    #[dust_dds(id=PID_DURABILITY as u32, optional)]
    pub(crate) durability: DurabilityQosPolicy,
    #[dust_dds(id=PID_DEADLINE as u32, optional)]
    pub(crate) deadline: DeadlineQosPolicy,
    #[dust_dds(id=PID_LATENCY_BUDGET as u32, optional)]
    pub(crate) latency_budget: LatencyBudgetQosPolicy,
    #[dust_dds(id=PID_LIVELINESS as u32, optional)]
    pub(crate) liveliness: LivelinessQosPolicy,
    #[dust_dds(id=PID_RELIABILITY as u32, optional, default_value=DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)]
    pub(crate) reliability: ReliabilityQosPolicy,
    #[dust_dds(id=PID_TRANSPORT_PRIORITY as u32, optional)]
    pub(crate) transport_priority: TransportPriorityQosPolicy,
    #[dust_dds(id=PID_LIFESPAN as u32, optional)]
    pub(crate) lifespan: LifespanQosPolicy,
    #[dust_dds(id=PID_DESTINATION_ORDER as u32, optional)]
    pub(crate) destination_order: DestinationOrderQosPolicy,
    #[dust_dds(id=PID_HISTORY as u32, optional)]
    pub(crate) history: HistoryQosPolicy,
    #[dust_dds(id=PID_RESOURCE_LIMITS as u32, optional)]
    pub(crate) resource_limits: ResourceLimitsQosPolicy,
    #[dust_dds(id=PID_OWNERSHIP as u32, optional)]
    pub(crate) ownership: OwnershipQosPolicy,
    #[dust_dds(id=PID_TOPIC_DATA as u32, optional)]
    pub(crate) topic_data: TopicDataQosPolicy,
    #[dust_dds(id=PID_DATA_REPRESENTATION as u32, optional)]
    pub(crate) representation: DataRepresentationQosPolicy,
}

impl TopicBuiltinTopicData {
    /// Get the key value of the discovered topic.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the name of the discovered topic.
    pub fn name(&self) -> &str {
        &self.name.value
    }

    /// Get the type name of the discovered topic.
    pub fn get_type_name(&self) -> &str {
        &self.type_name.value
    }

    /// Get the durability QoS policy of the discovered topic.
    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    /// Get the deadline QoS policy of the discovered topic.
    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    /// Get the latency budget QoS policy of the discovered topic.
    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    /// Get the liveliness QoS policy of the discovered topic.
    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    /// Get the reliability QoS policy of the discovered topic.
    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    /// Get the transport priority QoS policy of the discovered topic.
    pub fn transport_priority(&self) -> &TransportPriorityQosPolicy {
        &self.transport_priority
    }

    /// Get the lifespan QoS policy of the discovered topic.
    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    /// Get the destination order QoS policy of the discovered topic.
    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    /// Get the history QoS policy of the discovered topic.
    pub fn history(&self) -> &HistoryQosPolicy {
        &self.history
    }

    /// Get the resource limits QoS policy of the discovered topic.
    pub fn resource_limits(&self) -> &ResourceLimitsQosPolicy {
        &self.resource_limits
    }

    /// Get the ownership QoS policy of the discovered topic.
    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    /// Get the topic data QoS policy of the discovered topic.
    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    /// Get the data representation QoS policy of the discovered topic.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }
}

/// Structure representing a discovered [`DataWriter`](crate::publication::data_writer::DataWriter).
#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable")]
pub struct PublicationBuiltinTopicData {
    #[dust_dds(id=PID_ENDPOINT_GUID as u32, key)]
    pub(crate) key: BuiltInTopicKey,
    #[dust_dds(id=PID_PARTICIPANT_GUID as u32, key)]
    pub(crate) participant_key: BuiltInTopicKey,
    #[dust_dds(id=PID_TOPIC_NAME as u32)]
    pub(crate) topic_name: _String,
    #[dust_dds(id=PID_TYPE_NAME as u32)]
    pub(crate) type_name: _String,
    #[dust_dds(id=PID_TYPE_INFORMATION as u32, optional)]
    pub(crate) type_information: Option<TypeInformation>,
    #[dust_dds(id=PID_DURABILITY as u32, optional)]
    pub(crate) durability: DurabilityQosPolicy,
    #[dust_dds(id=PID_DEADLINE as u32, optional)]
    pub(crate) deadline: DeadlineQosPolicy,
    #[dust_dds(id=PID_LATENCY_BUDGET as u32, optional)]
    pub(crate) latency_budget: LatencyBudgetQosPolicy,
    #[dust_dds(id=PID_LIVELINESS as u32, optional)]
    pub(crate) liveliness: LivelinessQosPolicy,
    #[dust_dds(id=PID_RELIABILITY as u32, optional, default_value=DEFAULT_RELIABILITY_QOS_POLICY_DATA_WRITER)]
    pub(crate) reliability: ReliabilityQosPolicy,
    #[dust_dds(id=PID_LIFESPAN as u32, optional)]
    pub(crate) lifespan: LifespanQosPolicy,
    #[dust_dds(id=PID_USER_DATA as u32, optional)]
    pub(crate) user_data: UserDataQosPolicy,
    #[dust_dds(id=PID_OWNERSHIP as u32, optional)]
    pub(crate) ownership: OwnershipQosPolicy,
    #[dust_dds(id=PID_OWNERSHIP_STRENGTH as u32, optional)]
    pub(crate) ownership_strength: OwnershipStrengthQosPolicy,
    #[dust_dds(id=PID_DESTINATION_ORDER as u32, optional)]
    pub(crate) destination_order: DestinationOrderQosPolicy,
    #[dust_dds(id=PID_PRESENTATION as u32, optional)]
    pub(crate) presentation: PresentationQosPolicy,
    #[dust_dds(id=PID_PARTITION as u32, optional)]
    pub(crate) partition: PartitionQosPolicy,
    #[dust_dds(id=PID_TOPIC_DATA as u32, optional)]
    pub(crate) topic_data: TopicDataQosPolicy,
    #[dust_dds(id=PID_GROUP_DATA as u32, optional)]
    pub(crate) group_data: GroupDataQosPolicy,
    #[dust_dds(id=PID_DATA_REPRESENTATION as u32, optional)]
    pub(crate) representation: DataRepresentationQosPolicy,
    #[dust_dds(id=PID_ENDPOINT_SECURITY_PROTECTION_INFO as u32, optional)]
    pub(crate) protection_info: EndpointSecurityProtectionInfo,
    #[dust_dds(id=PID_ENDPOINT_SECURITY_SYMMETRIC_CIPHER_ALGORITHM_INFO as u32, optional)]
    pub(crate) symmetric_cipher: EndpointSecuritySymmetricCipherAlgorithmInfo,
}

impl PublicationBuiltinTopicData {
    /// Get the key value of the discovered writer.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the key value of the parent participant of the discovered writer.
    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    /// Get the name of the topic associated with the discovered writer.
    pub fn topic_name(&self) -> &str {
        &self.topic_name.value
    }

    /// Get the name of the type associated with the discovered writer.
    pub fn get_type_name(&self) -> &str {
        &self.type_name.value
    }

    /// Get the durability QoS policy of the discovered writer.
    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    /// Get the deadline QoS policy of the discovered writer.
    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    /// Get the latency budget QoS policy of the discovered writer.
    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    /// Get the liveliness QoS policy of the discovered writer.
    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    /// Get the reliability QoS policy of the discovered writer.
    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    /// Get the lifespan QoS policy of the discovered writer.
    pub fn lifespan(&self) -> &LifespanQosPolicy {
        &self.lifespan
    }

    /// Get the user data QoS policy of the discovered writer.
    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    /// Get the ownership QoS policy of the discovered writer.
    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    /// Get the ownership strength QoS policy of the discovered writer.
    pub fn ownership_strength(&self) -> &OwnershipStrengthQosPolicy {
        &self.ownership_strength
    }

    /// Get the destination order QoS policy of the discovered writer.
    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    /// Get the presentation QoS policy of the discovered writer.
    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    /// Get the partition QoS policy of the discovered writer.
    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    /// Get the topic data QoS policy of the topic associated with the discovered writer.
    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    /// Get the group data QoS policy of the discovered writer.
    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }

    /// Get the data representation QoS policy of the discovered writer.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }

    /// Get the protection info of the discovered writer.
    pub fn protection_info(&self) -> &EndpointSecurityProtectionInfo {
        &self.protection_info
    }

    /// Get the symmetric cipher algorithm info of the discovered writer.
    pub fn symmetric_cipher(&self) -> &EndpointSecuritySymmetricCipherAlgorithmInfo {
        &self.symmetric_cipher
    }
}

/// Structure representing a discovered [`DataReader`](crate::subscription::data_reader::DataReader).
#[derive(Debug, PartialEq, Eq, Clone, TypeSupport)]
#[dust_dds(extensibility = "mutable")]
pub struct SubscriptionBuiltinTopicData {
    #[dust_dds(id=PID_ENDPOINT_GUID as u32, key)]
    pub(crate) key: BuiltInTopicKey,
    #[dust_dds(id=PID_PARTICIPANT_GUID as u32, key)]
    pub(crate) participant_key: BuiltInTopicKey,
    #[dust_dds(id=PID_TOPIC_NAME as u32)]
    pub(crate) topic_name: _String,
    #[dust_dds(id=PID_TYPE_NAME as u32)]
    pub(crate) type_name: _String,
    #[dust_dds(id=PID_TYPE_INFORMATION as u32, optional)]
    pub(crate) type_information: Option<TypeInformation>,
    #[dust_dds(id=PID_DURABILITY as u32, optional)]
    pub(crate) durability: DurabilityQosPolicy,
    #[dust_dds(id=PID_DEADLINE as u32, optional)]
    pub(crate) deadline: DeadlineQosPolicy,
    #[dust_dds(id=PID_LATENCY_BUDGET as u32, optional)]
    pub(crate) latency_budget: LatencyBudgetQosPolicy,
    #[dust_dds(id=PID_LIVELINESS as u32, optional)]
    pub(crate) liveliness: LivelinessQosPolicy,
    #[dust_dds(id=PID_RELIABILITY as u32, optional, default_value=DEFAULT_RELIABILITY_QOS_POLICY_DATA_READER_AND_TOPICS)]
    pub(crate) reliability: ReliabilityQosPolicy,
    #[dust_dds(id=PID_OWNERSHIP as u32, optional)]
    pub(crate) ownership: OwnershipQosPolicy,
    #[dust_dds(id=PID_DESTINATION_ORDER as u32, optional)]
    pub(crate) destination_order: DestinationOrderQosPolicy,
    #[dust_dds(id=PID_USER_DATA as u32, optional)]
    pub(crate) user_data: UserDataQosPolicy,
    #[dust_dds(id=PID_TIME_BASED_FILTER as u32, optional)]
    pub(crate) time_based_filter: TimeBasedFilterQosPolicy,
    #[dust_dds(id=PID_PRESENTATION as u32, optional)]
    pub(crate) presentation: PresentationQosPolicy,
    #[dust_dds(id=PID_PARTITION as u32, optional)]
    pub(crate) partition: PartitionQosPolicy,
    #[dust_dds(id=PID_TOPIC_DATA as u32, optional)]
    pub(crate) topic_data: TopicDataQosPolicy,
    #[dust_dds(id=PID_GROUP_DATA as u32, optional)]
    pub(crate) group_data: GroupDataQosPolicy,
    #[dust_dds(id=PID_DATA_REPRESENTATION as u32, optional)]
    pub(crate) representation: DataRepresentationQosPolicy,
    #[dust_dds(id=PID_TYPE_CONSISTENCY_ENFORCEMENT as u32, optional)]
    pub(crate) type_consistency: TypeConsistencyEnforcementQosPolicy,
    #[dust_dds(id=PID_ENDPOINT_SECURITY_PROTECTION_INFO as u32, optional)]
    pub(crate) protection_info: EndpointSecurityProtectionInfo,
    #[dust_dds(id=PID_ENDPOINT_SECURITY_SYMMETRIC_CIPHER_ALGORITHM_INFO as u32, optional)]
    pub(crate) symmetric_cipher: EndpointSecuritySymmetricCipherAlgorithmInfo,
}

impl SubscriptionBuiltinTopicData {
    /// Get the key value of the discovered reader.
    pub fn key(&self) -> &BuiltInTopicKey {
        &self.key
    }

    /// Get the key value of the parent participant of the discovered reader.
    pub fn participant_key(&self) -> &BuiltInTopicKey {
        &self.participant_key
    }

    /// Get the name of the topic associated with the discovered reader.
    pub fn topic_name(&self) -> &str {
        &self.topic_name.value
    }

    /// Get the name of the type associated with the discovered reader.
    pub fn get_type_name(&self) -> &str {
        &self.type_name.value
    }

    /// Get the durability QoS policy of the discovered reader.
    pub fn durability(&self) -> &DurabilityQosPolicy {
        &self.durability
    }

    /// Get the deadline QoS policy of the discovered reader.
    pub fn deadline(&self) -> &DeadlineQosPolicy {
        &self.deadline
    }

    /// Get the latency budget QoS policy of the discovered reader.
    pub fn latency_budget(&self) -> &LatencyBudgetQosPolicy {
        &self.latency_budget
    }

    /// Get the liveliness QoS policy of the discovered reader.
    pub fn liveliness(&self) -> &LivelinessQosPolicy {
        &self.liveliness
    }

    /// Get the reliability QoS policy of the discovered reader.
    pub fn reliability(&self) -> &ReliabilityQosPolicy {
        &self.reliability
    }

    /// Get the ownership QoS policy of the discovered reader.
    pub fn ownership(&self) -> &OwnershipQosPolicy {
        &self.ownership
    }

    /// Get the destination order QoS policy of the discovered reader.
    pub fn destination_order(&self) -> &DestinationOrderQosPolicy {
        &self.destination_order
    }

    /// Get the user data QoS policy of the discovered reader.
    pub fn user_data(&self) -> &UserDataQosPolicy {
        &self.user_data
    }

    /// Get the time based filter QoS policy of the discovered reader.
    pub fn time_based_filter(&self) -> &TimeBasedFilterQosPolicy {
        &self.time_based_filter
    }

    /// Get the presentation QoS policy of the discovered reader.
    pub fn presentation(&self) -> &PresentationQosPolicy {
        &self.presentation
    }

    /// Get the partition QoS policy of the discovered reader.
    pub fn partition(&self) -> &PartitionQosPolicy {
        &self.partition
    }

    /// Get the topic data QoS policy of the topic associated with the discovered reader.
    pub fn topic_data(&self) -> &TopicDataQosPolicy {
        &self.topic_data
    }

    /// Get the group data QoS policy of the discovered reader.
    pub fn group_data(&self) -> &GroupDataQosPolicy {
        &self.group_data
    }

    /// Get the data representation QoS policy of the discovered reader.
    pub fn representation(&self) -> &DataRepresentationQosPolicy {
        &self.representation
    }

    /// Get the type consistency enforcement QoS policy of the discovered reader
    pub fn type_consistency(&self) -> &TypeConsistencyEnforcementQosPolicy {
        &self.type_consistency
    }

    /// Get the protection info of the discovered reader.
    pub fn protection_info(&self) -> &EndpointSecurityProtectionInfo {
        &self.protection_info
    }

    /// Get the symmetric cipher algorithm info of the discovered reader.
    pub fn symmetric_cipher(&self) -> &EndpointSecuritySymmetricCipherAlgorithmInfo {
        &self.symmetric_cipher
    }
}
