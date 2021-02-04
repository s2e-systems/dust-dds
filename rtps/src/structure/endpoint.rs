use crate::types::{ReliabilityKind, TopicKind, GUID};

use super::Entity;

pub struct Endpoint {
    pub entity: Entity,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
}

impl Endpoint {
    pub fn new(guid: GUID, topic_kind: TopicKind, reliability_level: ReliabilityKind) -> Self {
        let entity = Entity::new(guid);
        Self {
            entity,
            topic_kind,
            reliability_level,
        }
    }
}
