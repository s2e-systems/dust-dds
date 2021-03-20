use super::types::SubmessageKind;
use rust_rtps_pim::messages::types::SubmessageFlag;

pub mod acknack;
pub mod data;
pub mod data_frag;
pub mod gap;
pub mod heartbeat;
pub mod heartbeat_frag;
pub mod info_destination;
pub mod info_reply;
pub mod info_source;
pub mod info_timestamp;
pub mod nack_frag;
pub mod pad;

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
