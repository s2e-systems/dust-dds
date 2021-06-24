use super::{types::GUIDPIM, RTPSEntity};

pub trait RTPSGroup<PSM>: RTPSEntity<PSM>
where
    PSM: GUIDPIM,
{
}
