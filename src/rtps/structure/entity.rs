use crate::rtps::types::GUID;

pub struct RtpsEntity {
    pub guid: GUID,
}

impl RtpsEntity {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid
        }
    }
}