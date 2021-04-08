use crate::structure;

use super::types::GUID;

pub struct RTPSEntity<PSM: structure::Types> {
    pub guid: GUID<PSM>,
}

impl<PSM: structure::Types> RTPSEntity<PSM> {
    pub fn new(guid: GUID<PSM>) -> Self {
        Self { guid }
    }
}
