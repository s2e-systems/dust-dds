use rust_rtps_pim::structure::types::{EntityId, EntityKind, GuidPrefix, ProtocolVersion, GUID, Locator, LocatorKind, LocatorPort, LocatorAddress};
use rust_rtps_pim::behavior::types::Duration;
use rust_rtps_pim::messages::types::Count;

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "EntityKind")]
pub enum EntityKindDef {
    UserDefinedUnknown,
    BuiltInUnknown,
    BuiltInParticipant,
    UserDefinedWriterWithKey,
    BuiltInWriterWithKey,
    UserDefinedWriterNoKey,
    BuiltInWriterNoKey,
    UserDefinedReaderWithKey,
    BuiltInReaderWithKey,
    UserDefinedReaderNoKey,
    BuiltInReaderNoKey,
    UserDefinedWriterGroup,
    BuiltInWriterGroup,
    UserDefinedReaderGroup,
    BuiltInReaderGroup,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "EntityId")]
pub struct EntityIdDef {
    entity_key: [u8; 3],
    #[serde(with = "EntityKindDef")]
    entity_kind: EntityKind,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Duration")]
pub struct DurationDef {
    seconds: i32,
    fraction: u32,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "ProtocolVersion")]
pub struct ProtocolVersionDef {
    major: u8,
    minor: u8,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "GUID")]
pub struct GuidDef {
    #[serde(getter = "GUID::prefix")]
    prefix: GuidPrefix,
    #[serde(getter = "GUID::entity_id", with = "EntityIdDef")]
    entity_id: EntityId,
}
impl From<GuidDef> for GUID {
    fn from(def: GuidDef) -> GUID {
        GUID::new(def.prefix, def.entity_id)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Locator")]
pub struct LocatorDef {
    #[serde(getter = "Locator::kind")]
    kind: LocatorKind,
    #[serde(getter = "Locator::port")]
    port: LocatorPort,
    #[serde(getter = "Locator::address")]
    address: LocatorAddress,
}
impl From<LocatorDef> for Locator {
    fn from(def: LocatorDef) -> Locator {
        Locator::new(def.kind, def.port, def.address)
    }
}


#[derive(serde::Serialize, serde::Deserialize)]
#[serde(remote = "Count")]
pub struct CountDef(i32);

