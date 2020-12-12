use crate::rtps::types::GUID;

pub struct Entity {
    pub guid: GUID,
}

impl Entity {
    pub fn new(guid: GUID) -> Self {
        Self {
            guid
        }
    }
}