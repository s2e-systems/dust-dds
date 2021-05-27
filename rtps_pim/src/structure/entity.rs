use super::types::{EntityIdType, GUIDType, GuidPrefixType};

pub trait RTPSEntity<PSM: GuidPrefixType + EntityIdType + GUIDType<PSM>> {
    fn guid(&self) -> PSM::GUID;
}
