use rtps_pim::structure::{
    entity::{RtpsEntityAttributes, RtpsEntityConstructor},
    group::{RtpsGroupAttributes, RtpsGroupConstructor},
    types::Guid,
};

use super::rtps_entity_impl::RtpsEntityImpl;

pub struct RtpsGroupImpl {
    entity: RtpsEntityImpl,
}

impl RtpsGroupConstructor for RtpsGroupImpl {
    fn new(guid: Guid) -> Self {
        Self {
            entity: RtpsEntityImpl::new(guid),
        }
    }
}

impl RtpsEntityAttributes for RtpsGroupImpl {
    fn guid(&self) -> Guid {
        self.entity.guid()
    }
}

impl RtpsGroupAttributes for RtpsGroupImpl {}
