use crate::transport::types::Guid;

pub struct RtpsStatelessReader {
    guid: Guid,
}

impl RtpsStatelessReader {
    pub const fn new(guid: Guid) -> Self {
        Self { guid }
    }

    pub const fn guid(&self) -> Guid {
        self.guid
    }
}
