use super::types::Guid;

pub trait RtpsEntityConstructor {
    fn new(guid: Guid) -> Self;
}

pub trait RtpsEntityAttributes {
    fn guid(&self) -> Guid;
}
