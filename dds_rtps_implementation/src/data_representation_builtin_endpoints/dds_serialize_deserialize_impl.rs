use rust_dds_api::{
    dcps_psm::BuiltInTopicKey,
    infrastructure::qos_policy::{
        DeadlineQosPolicy, DestinationOrderQosPolicy, DestinationOrderQosPolicyKind,
        DurabilityQosPolicy, DurabilityQosPolicyKind, DurabilityServiceQosPolicy,
        GroupDataQosPolicy, HistoryQosPolicy, HistoryQosPolicyKind, LatencyBudgetQosPolicy,
        LifespanQosPolicy, LivelinessQosPolicy, LivelinessQosPolicyKind, OwnershipQosPolicy,
        OwnershipQosPolicyKind, OwnershipStrengthQosPolicy, PartitionQosPolicy,
        PresentationQosPolicy, PresentationQosPolicyAccessScopeKind, ReliabilityQosPolicy,
        ReliabilityQosPolicyKind, ResourceLimitsQosPolicy, TimeBasedFilterQosPolicy,
        TopicDataQosPolicy, TransportPriorityQosPolicy, UserDataQosPolicy,
    },
};
use rust_rtps_pim::{
    behavior::types::Duration,
    messages::types::Count,
    structure::types::{EntityId, EntityKind, Guid, GuidPrefix, Locator, ProtocolVersion},
};
use rust_rtps_psm::discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "UserDataQosPolicy")]
pub struct UserDataQosPolicyDef {
    pub value: Vec<u8>,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct UserDataQosPolicySerialize<'a>(
    #[serde(with = "UserDataQosPolicyDef")] pub &'a UserDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct UserDataQosPolicyDeserialize(
    #[serde(with = "UserDataQosPolicyDef")] pub UserDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Duration")]
pub struct DurationDef {
    seconds: i32,
    fraction: u32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DurationSerdeSerialize<'a>(#[serde(with = "DurationDef")] pub &'a Duration);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DurationSerdeDeserialize(#[serde(with = "DurationDef")] pub Duration);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Locator")]
pub struct LocatorDef {
    pub kind: i32,
    pub port: u32,
    pub address: [u8; 16],
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LocatorSerialize<'a>(#[serde(with = "LocatorDef")] pub &'a Locator);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LocatorDeserialize(#[serde(with = "LocatorDef")] pub Locator);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "ProtocolVersion")]
pub struct ProtocolVersionDef {
    pub major: u8,
    pub minor: u8,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ProtocolVersionSerdeSerialize<'a>(
    #[serde(with = "ProtocolVersionDef")] pub &'a ProtocolVersion,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct ProtocolVersionSerdeDeserialize(
    #[serde(with = "ProtocolVersionDef")] pub ProtocolVersion,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Guid")]
pub struct GuidDef {
    #[serde(with = "GuidPrefixDef")]
    pub prefix: GuidPrefix,
    #[serde(with = "EntityIdDef")]
    pub entity_id: EntityId,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct GuidSerdeSerialize<'a>(#[serde(with = "GuidDef")] pub &'a Guid);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct GuidSerdeDeserialize(#[serde(with = "GuidDef")] pub Guid);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "EntityId")]
pub struct EntityIdDef {
    pub entity_key: [u8; 3],
    pub entity_kind: EntityKind,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct EntityIdSerialize<'a>(#[serde(with = "EntityIdDef")] pub &'a EntityId);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct EntityIdDeserialize(#[serde(with = "EntityIdDef")] pub EntityId);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "GuidPrefix")]
pub struct GuidPrefixDef(pub [u8; 12]);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "BuiltinEndpointSet")]
pub struct BuiltinEndpointSetDef(pub u32);

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltinEndpointSetSerdeSerialize<'a>(
    #[serde(with = "BuiltinEndpointSetDef")] pub &'a BuiltinEndpointSet,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltinEndpointSetSerdeDeserialize(
    #[serde(with = "BuiltinEndpointSetDef")] pub BuiltinEndpointSet,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Count")]
pub struct CountDef(pub i32);

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct CountSerdeSerialize<'a>(#[serde(with = "CountDef")] pub &'a Count);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct CountSerdeDeserialize(#[serde(with = "CountDef")] pub Count);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "BuiltinEndpointQos")]
pub struct BuiltinEndpointQosDef(pub u32);

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltinEndpointQosSerdeSerialize<'a>(
    #[serde(with = "BuiltinEndpointQosDef")] pub &'a BuiltinEndpointQos,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltinEndpointQosSerdeDeserialize(
    #[serde(with = "BuiltinEndpointQosDef")] pub BuiltinEndpointQos,
);

type BuiltInTopicKeyTypeNative = i32;

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "BuiltInTopicKey")]
pub struct BuiltInTopicKeyDef {
    pub value: [BuiltInTopicKeyTypeNative; 4],
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltInTopicKeySerialize<'a>(
    #[serde(with = "BuiltInTopicKeyDef")] pub &'a BuiltInTopicKey,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltInTopicKeyDeserialize(#[serde(with = "BuiltInTopicKeyDef")] pub BuiltInTopicKey);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DurabilityQosPolicyKind")]
pub enum DurabilityQosPolicyKindDef {
    VolatileDurabilityQoS,
    TransientLocalDurabilityQoS,
    TransientDurabilityQoS,
    PersistentDurabilityQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DurabilityQosPolicy")]
pub struct DurabilityQosPolicyDef {
    #[serde(with = "DurabilityQosPolicyKindDef")]
    pub kind: DurabilityQosPolicyKind,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DurabilityQosPolicySerialize<'a>(
    #[serde(with = "DurabilityQosPolicyDef")] pub &'a DurabilityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DurabilityQosPolicyDeserialize(
    #[serde(with = "DurabilityQosPolicyDef")] pub DurabilityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "HistoryQosPolicyKind")]
pub enum HistoryQosPolicyKindDef {
    KeepLastHistoryQoS,
    KeepAllHistoryQos,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "rust_dds_api::dcps_psm::Duration")]
pub struct DcpsDurationDef {
    #[serde(getter = "rust_dds_api::dcps_psm::Duration::sec")]
    sec: i32,
    #[serde(getter = "rust_dds_api::dcps_psm::Duration::nanosec")]
    nanosec: u32,
}
impl From<DcpsDurationDef> for rust_dds_api::dcps_psm::Duration {
    fn from(def: DcpsDurationDef) -> rust_dds_api::dcps_psm::Duration {
        rust_dds_api::dcps_psm::Duration::new(def.sec, def.nanosec)
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DurabilityServiceQosPolicy")]
pub struct DurabilityServiceQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub service_cleanup_delay: rust_dds_api::dcps_psm::Duration,
    #[serde(with = "HistoryQosPolicyKindDef")]
    pub history_kind: HistoryQosPolicyKind,
    pub history_depth: i32,
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DurabilityServiceQosPolicySerialize<'a>(
    #[serde(with = "DurabilityServiceQosPolicyDef")] pub &'a DurabilityServiceQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DurabilityServiceQosPolicyDeserialize(
    #[serde(with = "DurabilityServiceQosPolicyDef")] pub DurabilityServiceQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DeadlineQosPolicy")]
pub struct DeadlineQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub period: rust_dds_api::dcps_psm::Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DeadlineQosPolicySerialize<'a>(
    #[serde(with = "DeadlineQosPolicyDef")] pub &'a DeadlineQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DeadlineQosPolicyDeserialize(
    #[serde(with = "DeadlineQosPolicyDef")] pub DeadlineQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LatencyBudgetQosPolicy")]
pub struct LatencyBudgetQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub duration: rust_dds_api::dcps_psm::Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LatencyBudgetQosPolicySerialize<'a>(
    #[serde(with = "LatencyBudgetQosPolicyDef")] pub &'a LatencyBudgetQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LatencyBudgetQosPolicyDeserialize(
    #[serde(with = "LatencyBudgetQosPolicyDef")] pub LatencyBudgetQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LivelinessQosPolicyKind")]
pub enum LivelinessQosPolicyKindDef {
    AutomaticLivelinessQoS,
    ManualByParticipantLivelinessQoS,
    ManualByTopicLivelinessQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LivelinessQosPolicy")]
pub struct LivelinessQosPolicyDef {
    #[serde(with = "LivelinessQosPolicyKindDef")]
    pub kind: LivelinessQosPolicyKind,
    #[serde(with = "DcpsDurationDef")]
    pub lease_duration: rust_dds_api::dcps_psm::Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LivelinessQosPolicySerialize<'a>(
    #[serde(with = "LivelinessQosPolicyDef")] pub &'a LivelinessQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LivelinessQosPolicyDeserialize(
    #[serde(with = "LivelinessQosPolicyDef")] pub LivelinessQosPolicy,
);

#[derive(Debug, PartialEq)]
pub enum ReliabilityQosPolicyKindDef {
    BestEffortReliabilityQos,
    ReliableReliabilityQos,
}

impl<'de> ReliabilityQosPolicyKindDef {
    pub fn serialize<S>(this: &ReliabilityQosPolicyKind, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match this {
            ReliabilityQosPolicyKind::BestEffortReliabilityQos => {
                serde::Serializer::serialize_unit_variant(
                    serializer,
                    "ReliabilityQosPolicyKindDef",
                    1u32,
                    "BestEffortReliabilityQos",
                )
            }
            ReliabilityQosPolicyKind::ReliableReliabilityQos => {
                serde::Serializer::serialize_unit_variant(
                    serializer,
                    "ReliabilityQosPolicyKindDef",
                    2u32,
                    "ReliableReliabilityQos",
                )
            }
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize)]
#[serde(remote = "ReliabilityQosPolicy")]
pub struct ReliabilityQosPolicyDef {
    #[serde(with = "ReliabilityQosPolicyKindDef")]
    pub kind: ReliabilityQosPolicyKind,
    #[serde(with = "DcpsDurationDef")]
    pub max_blocking_time: rust_dds_api::dcps_psm::Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ReliabilityQosPolicySerialize<'a>(
    #[serde(with = "ReliabilityQosPolicyDef")] pub &'a ReliabilityQosPolicy,
);

#[derive(Debug, PartialEq)]
pub struct ReliabilityQosPolicyDeserialize(
    pub ReliabilityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "TransportPriorityQosPolicy")]
pub struct TransportPriorityQosPolicyDef {
    pub value: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TransportPriorityQosPolicySerialize<'a>(
    #[serde(with = "TransportPriorityQosPolicyDef")] pub &'a TransportPriorityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct TransportPriorityQosPolicyDeserialize(
    #[serde(with = "TransportPriorityQosPolicyDef")] pub TransportPriorityQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "LifespanQosPolicy")]
pub struct LifespanQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub duration: rust_dds_api::dcps_psm::Duration,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct LifespanQosPolicySerialize<'a>(
    #[serde(with = "LifespanQosPolicyDef")] pub &'a LifespanQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LifespanQosPolicyDeserialize(
    #[serde(with = "LifespanQosPolicyDef")] pub LifespanQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DestinationOrderQosPolicyKind")]
pub enum DestinationOrderQosPolicyKindDef {
    ByReceptionTimestampDestinationOrderQoS,
    BySourceTimestampDestinationOrderQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "DestinationOrderQosPolicy")]
pub struct DestinationOrderQosPolicyDef {
    #[serde(with = "DestinationOrderQosPolicyKindDef")]
    pub kind: DestinationOrderQosPolicyKind,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DestinationOrderQosPolicySerialize<'a>(
    #[serde(with = "DestinationOrderQosPolicyDef")] pub &'a DestinationOrderQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct DestinationOrderQosPolicyDeserialize(
    #[serde(with = "DestinationOrderQosPolicyDef")] pub DestinationOrderQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "HistoryQosPolicy")]
pub struct HistoryQosPolicyDef {
    #[serde(with = "HistoryQosPolicyKindDef")]
    pub kind: HistoryQosPolicyKind,
    pub depth: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct HistoryQosPolicySerialize<'a>(
    #[serde(with = "HistoryQosPolicyDef")] pub &'a HistoryQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct HistoryQosPolicyDeserialize(#[serde(with = "HistoryQosPolicyDef")] pub HistoryQosPolicy);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "ResourceLimitsQosPolicy")]
pub struct ResourceLimitsQosPolicyDef {
    pub max_samples: i32,
    pub max_instances: i32,
    pub max_samples_per_instance: i32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ResourceLimitsQosPolicySerialize<'a>(
    #[serde(with = "ResourceLimitsQosPolicyDef")] pub &'a ResourceLimitsQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct ResourceLimitsQosPolicyDeserialize(
    #[serde(with = "ResourceLimitsQosPolicyDef")] pub ResourceLimitsQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "OwnershipQosPolicyKind")]
pub enum OwnershipQosPolicyKindDef {
    SharedOwnershipQoS,
    ExclusiveOwnershipQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "OwnershipQosPolicy")]
pub struct OwnershipQosPolicyDef {
    #[serde(with = "OwnershipQosPolicyKindDef")]
    pub kind: OwnershipQosPolicyKind,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct OwnershipQosPolicySerialize<'a>(
    #[serde(with = "OwnershipQosPolicyDef")] pub &'a OwnershipQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct OwnershipQosPolicyDeserialize(
    #[serde(with = "OwnershipQosPolicyDef")] pub OwnershipQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "TopicDataQosPolicy")]
pub struct TopicDataQosPolicyDef {
    pub value: Vec<u8>,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TopicDataQosPolicySerialize<'a>(
    #[serde(with = "TopicDataQosPolicyDef")] pub &'a TopicDataQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct TopicDataQosPolicyDeserialize(
    #[serde(with = "TopicDataQosPolicyDef")] pub TopicDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "OwnershipStrengthQosPolicy")]
pub struct OwnershipStrengthQosPolicyDef {
    pub value: i32,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct OwnershipStrengthQosPolicySerialize<'a>(
    #[serde(with = "OwnershipStrengthQosPolicyDef")] pub &'a OwnershipStrengthQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct OwnershipStrengthQosPolicyDeserialize(
    #[serde(with = "OwnershipStrengthQosPolicyDef")] pub OwnershipStrengthQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "PresentationQosPolicyAccessScopeKind")]
pub enum PresentationQosPolicyAccessScopeKindDef {
    InstancePresentationQoS,
    TopicPresentationQoS,
    GroupPresentationQoS,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "PresentationQosPolicy")]
pub struct PresentationQosPolicyDef {
    #[serde(with = "PresentationQosPolicyAccessScopeKindDef")]
    pub access_scope: PresentationQosPolicyAccessScopeKind,
    pub coherent_access: bool,
    pub ordered_access: bool,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PresentationQosPolicySerialize<'a>(
    #[serde(with = "PresentationQosPolicyDef")] pub &'a PresentationQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PresentationQosPolicyDeserialize(
    #[serde(with = "PresentationQosPolicyDef")] pub PresentationQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "PartitionQosPolicy")]
pub struct PartitionQosPolicyDef {
    pub name: String,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct PartitionQosPolicySerialize<'a>(
    #[serde(with = "PartitionQosPolicyDef")] pub &'a PartitionQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct PartitionQosPolicyDeserialize(
    #[serde(with = "PartitionQosPolicyDef")] pub PartitionQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "GroupDataQosPolicy")]
pub struct GroupDataQosPolicyDef {
    pub value: Vec<u8>,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct GroupDataQosPolicySerialize<'a>(
    #[serde(with = "GroupDataQosPolicyDef")] pub &'a GroupDataQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct GroupDataQosPolicyDeserialize(
    #[serde(with = "GroupDataQosPolicyDef")] pub GroupDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "TimeBasedFilterQosPolicy")]
pub struct TimeBasedFilterQosPolicyDef {
    #[serde(with = "DcpsDurationDef")]
    pub minimum_separation: rust_dds_api::dcps_psm::Duration,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct TimeBasedFilterQosPolicySerialize<'a>(
    #[serde(with = "TimeBasedFilterQosPolicyDef")] pub &'a TimeBasedFilterQosPolicy,
);
#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct TimeBasedFilterQosPolicyDeserialize(
    #[serde(with = "TimeBasedFilterQosPolicyDef")] pub TimeBasedFilterQosPolicy,
);
