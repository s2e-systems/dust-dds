use rust_rtps_pim::structure::types::{EntityId, EntityKind};
use rust_rtps_pim::behavior::types::Duration;

#[derive(serde::Serialize)]
#[serde(remote = "EntityKind")]
enum EntityKindDef {
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

#[derive(serde::Serialize)]
#[serde(remote = "EntityId")]
struct EntityIdDef {
    entity_key: [u8; 3],
    #[serde(with = "EntityKindDef")]
    entity_kind: EntityKind,
}

#[derive(serde::Serialize)]
#[serde(remote = "Duration")]
struct DurationDef {
    seconds: i32,
    fraction: u32,
}