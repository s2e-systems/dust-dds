use super::{
    types::{EntityIdType, GuidPrefixType},
    RTPSEntity,
};

pub trait RTPSGroup<PSM: GuidPrefixType + EntityIdType>: RTPSEntity<PSM> {}
