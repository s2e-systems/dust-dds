use super::types::Guid;

pub struct RtpsEntity {
    guid: Guid,
}

impl RtpsEntity {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl RtpsEntity {
    pub fn guid(&self) -> Guid {
        self.guid
    }
}
