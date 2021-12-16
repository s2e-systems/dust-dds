use super::{
    entity::RtpsEntity,
    types::{Guid, Locator, ReliabilityKind, TopicKind},
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

pub trait RtpsEndpointAttributes {
    fn topic_kind(&self) -> &TopicKind;
    fn reliability_level(&self) -> &ReliabilityKind;
    fn unicast_locator_list(&self) -> &[Locator];
    fn multicast_locator_list(&self) -> &[Locator];
}
