use crate::types::GUID;

pub trait Entity {
    fn guid(&self) -> GUID;
}
