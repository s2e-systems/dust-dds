use rust_rtps_pim::messages::types::SubmessageFlag;

use super::types::SubmessageKind;

pub mod info_timestamp;

pub struct SubmessageHeader {
    submessage_id: <Self as rust_rtps_pim::messages::submessages::SubmessageHeader>::SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: u16,
}

impl rust_rtps_pim::messages::submessages::SubmessageHeader for SubmessageHeader {
    type SubmessageKind = SubmessageKind;

    fn submessage_id(&self) -> &Self::SubmessageKind {
        &self.submessage_id
    }

    fn flags(&self) -> &[SubmessageFlag; 8] {
        &self.flags
    }

    fn submessage_length(&self) -> &u16 {
        &self.submessage_length
    }
}
