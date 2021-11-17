use rust_rtps_pim::{
    behavior::types::Duration,
    messages::types::Count,
    structure::types::{EntityId, EntityKind, Guid, GuidPrefix, Locator, ProtocolVersion},
};

use super::parameter_id_values::{DEFAULT_DOMAIN_TAG, DEFAULT_EXPECTS_INLINE_QOS, DEFAULT_PARTICIPANT_LEASE_DURATION};


#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Duration")]
pub struct DurationDef {
    seconds: i32,
    fraction: u32,
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct DurationSerialize<'a>(#[serde(with = "DurationDef")] pub &'a Duration);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct DurationDeserialize(#[serde(with = "DurationDef")] pub Duration);
impl Default for DurationDeserialize {
    fn default() -> Self {
        Self(DEFAULT_PARTICIPANT_LEASE_DURATION)
    }
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Locator")]
pub struct LocatorDef {
    pub kind: i32,
    pub port: u32,
    pub address: [u8; 16],
}
#[derive(Debug, PartialEq, serde::Serialize, derive_more::From)]
pub struct LocatorSerialize<'a>(#[serde(with = "LocatorDef")] pub &'a Locator);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct LocatorDeserialize(#[serde(with = "LocatorDef")] pub Locator);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "ProtocolVersion")]
pub struct ProtocolVersionDef {
    pub major: u8,
    pub minor: u8,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct ProtocolVersionSerialize<'a>(
    #[serde(with = "ProtocolVersionDef")] pub &'a ProtocolVersion,
);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct ProtocolVersionDeserialize(
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
pub struct GuidSerialize<'a>(#[serde(with = "GuidDef")] pub &'a Guid);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct GuidDeserialize(#[serde(with = "GuidDef")] pub Guid);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "EntityId")]
pub struct EntityIdDef {
    pub entity_key: [u8; 3],
    pub entity_kind: EntityKind,
}
#[derive(Debug, PartialEq, serde::Serialize)]
pub struct EntityIdSerialize<'a>(#[serde(with = "EntityIdDef")] pub &'a EntityId);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct EntityIdDeserialize(#[serde(with = "EntityIdDef")] pub EntityId);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "GuidPrefix")]
pub struct GuidPrefixDef(pub [u8; 12]);


#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(remote = "Count")]
pub struct CountDef(pub i32);

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct CountSerdeSerialize<'a>(#[serde(with = "CountDef")] pub &'a Count);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct CountDeserialize(#[serde(with = "CountDef")] pub Count);

#[derive(Debug, PartialEq, serde::Serialize, derive_more::From)]
pub struct ExpectsInclineQosSerialize<'a>(pub &'a bool);
impl Default for ExpectsInclineQosSerialize<'_> {
    fn default() -> Self {
        Self(&DEFAULT_EXPECTS_INLINE_QOS)
    }
}
#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct ExpectsInclineQosDeserialize(pub bool);
impl Default for ExpectsInclineQosDeserialize {
    fn default() -> Self {
        Self(DEFAULT_EXPECTS_INLINE_QOS)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, derive_more::From)]
pub struct DomainTag<'a>(pub &'a str);
impl<'a> Default for DomainTag<'a> {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG)
    }
}

#[derive(Debug, PartialEq, serde::Serialize, derive_more::From)]
pub struct DomainTagSerialize<'a>(pub &'a DomainTag<'a>);

#[derive(Debug, PartialEq, serde::Deserialize, derive_more::Into)]
pub struct DomainTagDeserialize(pub String);
impl Default for DomainTagDeserialize {
    fn default() -> Self {
        Self(DEFAULT_DOMAIN_TAG.to_string())
    }
}