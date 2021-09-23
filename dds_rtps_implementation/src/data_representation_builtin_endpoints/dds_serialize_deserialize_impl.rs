use rust_dds_api::infrastructure::qos_policy::UserDataQosPolicy;
use rust_rtps_pim::{
    behavior::types::Duration,
    discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet},
    messages::types::Count,
    structure::types::{EntityId, EntityKind, Guid, GuidPrefix, Locator, ProtocolVersion},
};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "UserDataQosPolicy")]
pub struct UserDataQosPolicyDef<'a> {
    pub value: &'a [u8],
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct UserDataQosPolicySerdeSerialize<'a>(
    #[serde(with = "UserDataQosPolicyDef")] pub &'a UserDataQosPolicy<'a>,
);

#[derive(Debug, PartialEq, serde::Deserialize)]
pub struct UserDataQosPolicySerdeDeserialize<'a>(
    #[serde(with = "UserDataQosPolicyDef")]
    #[serde(borrow)]
    pub UserDataQosPolicy<'a>,
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
