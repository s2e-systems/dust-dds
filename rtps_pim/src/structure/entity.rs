use crate::RtpsPim;

pub struct RTPSEntity<PSM: RtpsPim> {
    guid: PSM::Guid,
}

impl<PSM: RtpsPim> RTPSEntity<PSM> {
    pub fn new(guid: PSM::Guid) -> Self {
        Self { guid }
    }

    pub fn guid(&self) -> &PSM::Guid {
        &self.guid
    }
}
