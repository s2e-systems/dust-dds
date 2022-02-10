use rust_rtps_pim::structure::{types::Guid, entity::RtpsEntityAttributes};

pub struct RtpsEntityImpl {
    guid: Guid,
}

impl RtpsEntityImpl {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl RtpsEntityAttributes for RtpsEntityImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}
