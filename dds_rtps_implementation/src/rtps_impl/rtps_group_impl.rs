use rust_rtps_pim::structure::types::Guid;

pub struct RtpsGroupImpl {
    guid: Guid,
}

impl RtpsGroupImpl {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl rust_rtps_pim::structure::RtpsGroup for RtpsGroupImpl {}

impl rust_rtps_pim::structure::RtpsEntity for RtpsGroupImpl {
    fn guid(&self) -> &Guid {
        &self.guid
    }
}
