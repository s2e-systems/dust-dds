use super::types::GUID;

pub trait RtpsEntity {
    fn guid(&self) -> &GUID;
}
