use rtps_pim::structure::types::Guid;

use super::rtps_entity_impl::RtpsEntityImpl;

pub struct RtpsGroupImpl {
    entity: RtpsEntityImpl,
}

impl RtpsGroupImpl {
    pub fn new(guid: Guid) -> Self {
        Self {
            entity: RtpsEntityImpl::new(guid),
        }
    }
}

impl RtpsGroupImpl {
    pub fn guid(&self) -> Guid {
        self.entity.guid()
    }
}
