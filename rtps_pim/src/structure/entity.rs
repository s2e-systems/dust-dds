use crate::PIM;

use super::types::GUID;

pub trait RTPSEntity<PSM: PIM> {
    fn guid(&self) -> GUID<PSM>;
}
