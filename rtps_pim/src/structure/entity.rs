use super::types::GUIDPIM;

pub trait RTPSEntity<PSM> {
    fn guid(&self) -> &PSM::GUIDType
    where
        PSM: GUIDPIM<PSM>;
}
