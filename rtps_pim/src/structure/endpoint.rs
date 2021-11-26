use super::{
    entity::RtpsEntity,
    types::{Guid, ReliabilityKind, TopicKind},
};

pub struct RtpsEndpoint<L> {
    pub entity: RtpsEntity,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: L,
    pub multicast_locator_list: L,
}

impl<L> RtpsEndpoint<L> {
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: L,
        multicast_locator_list: L,
    ) -> Self {
        Self {
            entity: RtpsEntity::new(guid),
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
        }
    }
}
