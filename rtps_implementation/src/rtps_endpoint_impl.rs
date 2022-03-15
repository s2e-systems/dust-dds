use rtps_pim::structure::{
    endpoint::RtpsEndpointAttributes,
    entity::RtpsEntityAttributes,
    types::{Guid, Locator, ReliabilityKind, TopicKind},
};

use super::rtps_entity_impl::RtpsEntityImpl;

pub struct RtpsEndpointImpl {
    pub entity: RtpsEntityImpl,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
}

impl RtpsEndpointImpl {
    pub fn new(
        guid: Guid,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: &[Locator],
        multicast_locator_list: &[Locator],
    ) -> Self {
        Self {
            entity: RtpsEntityImpl { guid },
            topic_kind,
            reliability_level,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
        }
    }
}

impl RtpsEntityAttributes for RtpsEndpointImpl {
    fn guid(&self) -> Guid {
        self.entity.guid
    }
}

impl RtpsEndpointAttributes for RtpsEndpointImpl {
    fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    fn reliability_level(&self) -> ReliabilityKind {
        self.reliability_level
    }

    fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }
}
