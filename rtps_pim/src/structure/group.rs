use super::{
    types::{EntityIdType, GUIDType, GuidPrefixType},
    RTPSEntity,
};

pub trait RTPSGroup<PSM: GuidPrefixType + EntityIdType + GUIDType<PSM>>: RTPSEntity<PSM> {}
