use super::types::{EntityIdPIM, GUIDPIM, GuidPrefixPIM};

pub trait RTPSEntity<PSM: GuidPrefixPIM + EntityIdPIM + GUIDPIM<PSM>> {
    fn guid(&self) -> &PSM::GUIDType;
}
