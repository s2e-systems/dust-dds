use rust_dds_api::{dcps_psm::BuiltInTopicKey, infrastructure::qos_policy::{DurabilityQosPolicy, DurabilityQosPolicyKind, DurabilityServiceQosPolicy, HistoryQosPolicyKind, UserDataQosPolicy}};
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
pub struct UserDataQosPolicySerdeSerialize<'a>(
    #[serde(with = "UserDataQosPolicyDef")] pub &'a UserDataQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct UserDataQosPolicySerdeDeserialize(
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
pub struct LocatorSerdeSerialize<'a>(#[serde(with = "LocatorDef")] pub &'a Locator);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct LocatorSerdeDeserialize(#[serde(with = "LocatorDef")] pub Locator);

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
pub struct BuiltInTopicKeyDef{
    pub value: [BuiltInTopicKeyTypeNative; 4],
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct BuiltInTopicKeySerialize<'a>(
    #[serde(with = "BuiltInTopicKeyDef")] pub &'a BuiltInTopicKey,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct BuiltInTopicKeyDeserialize(
    #[serde(with = "BuiltInTopicKeyDef")] pub BuiltInTopicKey,
);

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
    #[serde(with = "DurabilityServiceQosPolicyDef")] pub &'a  DurabilityServiceQosPolicy,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct  DurabilityServiceQosPolicyDeserialize(
    #[serde(with = "DurabilityServiceQosPolicyDef")] pub  DurabilityServiceQosPolicy,
);