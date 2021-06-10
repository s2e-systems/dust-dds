use super::types::{EntityIdPIM, GuidPrefixPIM, GUIDPIM};

pub trait RTPSEntity<PSM: GuidPrefixPIM + EntityIdPIM + GUIDPIM> {
    fn guid(&self) -> &PSM::GUIDType;
}
