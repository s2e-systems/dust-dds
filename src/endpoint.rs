use crate::types::{TopicKind, ReliabilityKind, LocatorList, GUID};
use crate::entity::{Entity};

pub struct Endpoint{
    pub entity : Entity,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: LocatorList,
    pub multicast_locator_list: LocatorList,
}


#[cfg(test)]
mod tests {
    use super::*;

}

