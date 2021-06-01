use super::types::{EntityIdPIM, GUIDType, GuidPrefixPIM};

pub trait RTPSEntity<PSM: GuidPrefixPIM + EntityIdPIM + GUIDType<PSM>> {
    fn guid(&self) -> &PSM::GUID;
}
