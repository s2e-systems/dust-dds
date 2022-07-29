use super::types::Guid;

pub struct RtpsEntityImpl {
    guid: Guid,
}

impl RtpsEntityImpl {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

impl RtpsEntityImpl {
    pub fn guid(&self) -> Guid {
        self.guid
    }
}
