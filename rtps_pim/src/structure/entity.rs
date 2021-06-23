use super::types::GUIDPIM;

pub trait RTPSEntity<PSM>
where
    PSM: GUIDPIM<PSM>,
{
    fn guid(&self) -> &PSM::GUIDType;
}
