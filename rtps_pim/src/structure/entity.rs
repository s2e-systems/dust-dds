use super::types::GUID;

pub trait RTPSEntity {
    fn guid(&self) -> &GUID;
}
