use crate::types::{EntityId, GuidPrefix};

use super::RTPSEntity;

pub struct RTPSGroup<GuidPrefixType: GuidPrefix, EntityIdType: EntityId> {
    pub entity: RTPSEntity<GuidPrefixType, EntityIdType>,
}

impl<GuidPrefixType: GuidPrefix, EntityIdType: EntityId> core::ops::Deref
    for RTPSGroup<GuidPrefixType, EntityIdType>
{
    type Target = RTPSEntity<GuidPrefixType, EntityIdType>;

    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}
