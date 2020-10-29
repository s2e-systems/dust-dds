use crate::types::GUID;

pub trait RtpsEntity {
    fn guid(&self) -> GUID;
}