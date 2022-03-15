use rtps_pim::structure::{entity::RtpsEntityAttributes, types::Guid};

pub struct RtpsEntityImpl {
    pub guid: Guid,
}

impl RtpsEntityAttributes for RtpsEntityImpl {
    fn guid(&self) -> Guid {
        self.guid
    }
}
