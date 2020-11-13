use crate::types::GUID;

pub trait RtpsEntity: 'static + Send + Sync{
    fn guid(&self) -> GUID;
}