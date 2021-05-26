use super::types::{EntityIdType, GuidPrefixType, GUID};

pub trait RTPSEntity<PSM: GuidPrefixType + EntityIdType> {
    fn guid(&self) -> GUID<PSM>;
}
