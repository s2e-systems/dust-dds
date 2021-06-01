use super::{
    types::{EntityIdPIM, GUIDPIM, GuidPrefixPIM},
    RTPSEntity,
};

pub trait RTPSGroup<PSM: GuidPrefixPIM + EntityIdPIM + GUIDPIM<PSM>>: RTPSEntity<PSM> {}
