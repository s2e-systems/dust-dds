use rust_rtps_pim::structure::{types::Guid, entity::RtpsEntityAttributes};

pub struct RtpsEntityImpl {
    pub guid: Guid,
}

impl RtpsEntityAttributes for RtpsEntityImpl {
    fn guid(&self) -> Guid {
        self.guid
    }
}
