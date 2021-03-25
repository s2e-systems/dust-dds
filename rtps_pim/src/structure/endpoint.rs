use crate::types::{EntityId, GuidPrefix, Locator, ReliabilityKind, TopicKind};

use super::RTPSEntity;

pub struct RTPSEndpoint<
    GuidPrefixType: GuidPrefix,
    EntityIdType: EntityId,
    LocatorType: Locator,
    LocatorList: IntoIterator<Item = LocatorType>,
> {
    pub entity: RTPSEntity<GuidPrefixType, EntityIdType>,
    pub topic_kind: TopicKind,
    pub reliability_level: ReliabilityKind,
    pub unicast_locator_list: LocatorList,
    pub multicast_locator_list: LocatorList,
}

impl<
GuidPrefixType: GuidPrefix,
EntityIdType: EntityId,
LocatorType: Locator,
LocatorList: IntoIterator<Item = LocatorType>,
> core::ops::Deref for RTPSEndpoint<GuidPrefixType, EntityIdType, LocatorType, LocatorList> {
    type Target = RTPSEntity<GuidPrefixType, EntityIdType>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}