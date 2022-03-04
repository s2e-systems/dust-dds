use rust_rtps_pim::structure::{
    entity::RtpsEntityAttributes,
    group::{RtpsGroupAttributes, RtpsGroupConstructor},
    types::Guid,
};

use super::rtps_entity_impl::RtpsEntityImpl;

pub struct RtpsGroupImpl {
    pub entity: RtpsEntityImpl,
}

impl RtpsGroupConstructor for RtpsGroupImpl {
    fn new(guid: Guid) -> Self {
        Self { entity: RtpsEntityImpl { guid } }
    }
}

impl RtpsEntityAttributes for RtpsGroupImpl {
    fn guid(&self) -> Guid {
        self.entity.guid
    }
}

impl RtpsGroupAttributes for RtpsGroupImpl { }
