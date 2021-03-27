use crate::RtpsPsm;

pub struct RTPSEntity<PSM: RtpsPsm> {
    pub guid: PSM::Guid,
}
