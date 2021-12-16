use super::types::Guid;

pub struct RtpsEntity {
    pub guid: Guid,
}

impl RtpsEntity {
    pub fn new(guid: Guid) -> Self {
        Self { guid }
    }
}

pub trait RtpsEntityAttributes {
    fn guid(&self) -> &Guid;
}