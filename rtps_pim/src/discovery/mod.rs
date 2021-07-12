use crate::structure::types::EntityId;

// pub mod types;

// pub mod spdp_data;
// pub mod spdp_endpoints;
// pub mod sedp_endpoints;

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
    entity_key: [0x00, 0x01, 0x00],
    entity_kind: crate::structure::types::EntityKind::BuiltInWriterWithKey,
};
