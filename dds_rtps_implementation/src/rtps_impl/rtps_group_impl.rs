use rust_rtps_pim::structure::types::Guid;

pub struct RtpsGroupImpl {
    guid: Guid,
}

impl RtpsGroupImpl {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}
