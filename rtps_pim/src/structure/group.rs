use super::types::Guid;

pub trait RtpsGroupAttributes {}

pub trait RtpsGroupConstructor {
    fn new(guid: Guid) -> Self;
}
