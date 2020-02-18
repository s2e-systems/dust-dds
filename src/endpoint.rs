use crate::entity::Entity;
use crate::types::{LocatorList, ReliabilityKind, TopicKind, GUID};

/// RTPS Endpoint represents the possible communication endpoints from the point of view of the RTPS protocol.
/// There are two kinds of RTPS Endpoint entities: Writer endpoints and Reader endpoints. The Endpoint is described
/// in section 8.2.6 The RTPS Endpoint of the RTPS OMG standard.
pub struct Endpoint {
    /// Entity base class (contains the GUID)
    entity: Entity,
    /// Used to indicate whether the Endpoint supports instance lifecycle management operations. Indicates whether the Endpoint is associated with a DataType that has defined some fields as containing the DDS key.
    topic_kind: TopicKind,
    /// The level of reliability supported by the Endpoint.
    reliability_level: ReliabilityKind,
    /// List of unicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty
    unicast_locator_list: LocatorList,
    /// List of multicast locators (transport, address, port combinations) that can be used to send messages to the Endpoint. The list may be empty.
    multicast_locator_list: LocatorList,
}

impl Endpoint {
    pub fn new(
        entity: Entity,
        topic_kind: TopicKind,
        reliability_level: ReliabilityKind,
        unicast_locator_list: LocatorList,
        multicast_locator_list: LocatorList,
    ) -> Self {
        Endpoint {
            entity,
            topic_kind,
            reliability_level,
            unicast_locator_list,
            multicast_locator_list,
        }
    }

    pub fn guid(&self) -> &GUID {
        &self.entity.guid
    }

    pub fn topic_kind(&self) -> &TopicKind {
        &self.topic_kind
    }

    pub fn reliability_level(&self) -> &ReliabilityKind {
        &self.reliability_level
    }

    pub fn unicast_locator_list(&self) -> &LocatorList {
        &self.unicast_locator_list
    }

    pub fn multicast_locator_list(&self) -> &LocatorList {
        &self.multicast_locator_list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
