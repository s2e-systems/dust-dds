use super::{
    types::{EntityIdPIM, GuidPrefixPIM, GUIDPIM},
    RTPSEntity,
};

pub trait RTPSGroup<PSM: GuidPrefixPIM + EntityIdPIM + GUIDPIM>: RTPSEntity<PSM> {}
