use super::{
    types::{EntityIdPIM, GUIDType, GuidPrefixPIM},
    RTPSEntity,
};

pub trait RTPSGroup<PSM: GuidPrefixPIM + EntityIdPIM + GUIDType<PSM>>: RTPSEntity<PSM> {}
