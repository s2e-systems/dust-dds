use crate::RtpsPim;

pub struct RTPSEntity<PSM: RtpsPim> {
    pub guid: PSM::Guid,
}
