use crate::structure::types::{EntityId, EntityKind};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0, 0x02],
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0, 0x02],
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0, 0x03],
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0, 0x03],
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: [0, 0, 0x04],
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
    entity_key: [0, 0, 0x04],
    entity_kind: EntityKind::BuiltInReaderWithKey,
};
