use super::{
    types::{EntityIdPIM, GuidPrefixPIM, GUIDPIM},
    RTPSEntity,
};

pub trait RTPSGroup<PSM: GuidPrefixPIM + EntityIdPIM + GUIDPIM<PSM>>: RTPSEntity<PSM> {}
