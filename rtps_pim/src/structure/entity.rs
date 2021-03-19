use crate::types::GUID;

pub trait RTPSEntity {
    fn guid(&self) -> GUID;
}
