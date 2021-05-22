use crate::structure;

use super::types::GUID;

pub trait RTPSEntity<PSM: structure::Types> {
    fn guid(&self) -> GUID<PSM>;
}
