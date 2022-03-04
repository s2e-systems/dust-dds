use super::types::Guid;

pub trait RtpsEntityAttributes {
    fn guid(&self) -> Guid;
}
