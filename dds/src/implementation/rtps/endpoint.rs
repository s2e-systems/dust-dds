use rtps_pim::structure::types::{Guid, Locator, ReliabilityKind, TopicKind};

use super::entity::RtpsEntityImpl;

pub struct RtpsEndpointImpl {
    entity: RtpsEntityImpl,
    topic_kind: TopicKind,
    reliability_level: ReliabilityKind,
    unicast_locator_list: Vec<Locator>,
    multicast_locator_list: Vec<Locator>,
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
            entity: RtpsEntityImpl::new(guid),
            topic_kind,
            reliability_level,
            unicast_locator_list: unicast_locator_list.to_vec(),
            multicast_locator_list: multicast_locator_list.to_vec(),
        }
    }
}

impl RtpsEndpointImpl {
    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }
}

impl RtpsEndpointImpl {
    pub fn topic_kind(&self) -> TopicKind {
        self.topic_kind
    }

    pub fn reliability_level(&self) -> ReliabilityKind {
        self.reliability_level
    }

    pub fn unicast_locator_list(&self) -> &[Locator] {
        &self.unicast_locator_list
    }

    pub fn multicast_locator_list(&self) -> &[Locator] {
        &self.multicast_locator_list
    }
}
