use crate::structure;

pub struct RTPSEntity<PSM: structure::Types> {
    pub guid: PSM::Guid,
}

impl<PSM: structure::Types> RTPSEntity<PSM> {
    pub fn new(guid: PSM::Guid) -> Self {
        Self { guid }
    }
}
