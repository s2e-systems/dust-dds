use crate::types::{Locator, ReliabilityKind, TopicKind, GUID};

use super::Entity;

pub struct Endpoint {
    pub entity: Entity,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
}

impl Endpoint {
    pub fn new(
        guid: GUID,
        unicast_locator_list: Vec<Locator>,
        multicast_locator_list: Vec<Locator>,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
    ) -> Self {
        let entity = Entity::new(guid);
        Self {
            entity,
            unicast_locator_list,
            multicast_locator_list,
            topic_kind,
            reliability_level,
        }
    }
}
