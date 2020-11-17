use crate::types::GUID;
use crate::structure::RtpsEntity;

pub struct RtpsGroup {
    guid: GUID,
}

impl RtpsGroup {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid,
        }
    }
}

impl RtpsEntity for RtpsGroup {
    fn guid(&self) -> GUID {
        self.guid
    }
}