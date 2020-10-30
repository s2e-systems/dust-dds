use crate::types::GUID;

pub trait RtpsEntity: 'static {
    fn guid(&self) -> GUID;
}