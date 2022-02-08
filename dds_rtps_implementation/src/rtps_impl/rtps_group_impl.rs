use rust_rtps_pim::structure::{
    entity::RtpsEntityAttributes, group::{RtpsGroupAttributes, RtpsGroupConstructor}, types::Guid,
};

pub struct RtpsGroupImpl {
    guid: Guid,
}

impl RtpsGroupConstructor for RtpsGroupImpl {
    fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl RtpsEntityAttributes for RtpsGroupImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}

impl RtpsGroupAttributes for RtpsGroupImpl { }
