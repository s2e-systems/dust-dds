use crate::transport::types::Guid;

pub struct RtpsStatelessReader {
    guid: Guid,
}

impl RtpsStatelessReader {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }

    pub fn guid(&self) -> Guid {
        self.guid
    }
}
