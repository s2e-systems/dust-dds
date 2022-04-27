use rtps_pim::structure::{
    entity::{RtpsEntityAttributes, RtpsEntityConstructor},
    types::Guid,
};

pub struct RtpsEntityImpl {
    guid: Guid,
}

impl RtpsEntityConstructor for RtpsEntityImpl {
    fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl RtpsEntityAttributes for RtpsEntityImpl {
    fn guid(&self) -> Guid {
        self.guid
    }
}
