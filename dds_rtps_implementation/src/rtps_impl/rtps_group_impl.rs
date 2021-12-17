use rust_rtps_pim::structure::{
    entity::RtpsEntityAttributes, group::RtpsGroupAttributes, types::Guid,
};

pub struct RtpsGroupImpl {
    guid: Guid,
}

impl RtpsGroupImpl {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl RtpsEntityAttributes for RtpsGroupImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}

impl RtpsGroupAttributes for RtpsGroupImpl {}
