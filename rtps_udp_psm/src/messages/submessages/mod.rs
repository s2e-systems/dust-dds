use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::types::UShort;

use super::types::SubmessageKind;

pub mod info_timestamp;

pub struct SubmessageHeader {
    submessage_id: <Self as rust_rtps_pim::messages::submessages::SubmessageHeader>::SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: UShort,
}

impl rust_rtps_pim::messages::submessages::SubmessageHeader for SubmessageHeader {
    type SubmessageKind = SubmessageKind;
    type UShort = UShort;

    fn submessage_id(&self) -> &Self::SubmessageKind {
        &self.submessage_id
    }

    fn flags(&self) -> &[SubmessageFlag; 8] {
        &self.flags
    }

    fn submessage_length(&self) -> &Self::UShort {
        &self.submessage_length
    }
}
