use super::types::Guid;

pub trait RtpsEntity {
    fn guid(&self) -> &Guid;
}
