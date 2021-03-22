use super::types::{SubmessageFlag, SubmessageKind};
// use rust_rtps_pim::messages::types::SubmessageFlag;

pub mod acknack;
// pub mod data;
// pub mod data_frag;
// pub mod gap;
// pub mod heartbeat;
// pub mod heartbeat_frag;
// pub mod info_destination;
// pub mod info_reply;
// pub mod info_source;
// pub mod info_timestamp;
// pub mod nack_frag;
// pub mod pad;

#[derive(Clone, Copy)]
pub struct SubmessageHeader {
    submessage_id: <Self as rust_rtps_pim::messages::submessages::SubmessageHeader>::SubmessageKind,
    flags: u8,
    submessage_length: u16,
}

impl rust_rtps_pim::messages::submessages::SubmessageHeader for SubmessageHeader {
    type SubmessageKind = SubmessageKind;
    type SubmessageFlag = SubmessageFlag;

    fn submessage_id(&self) -> Self::SubmessageKind {
        self.submessage_id
    }

    fn flags(&self) -> [Self::SubmessageFlag; 8] {
        [
            (self.flags & 1 == 1).into(),
            (self.flags & 2 == 2).into(),
            (self.flags & 4 == 4).into(),
            (self.flags & 8 == 8).into(),
            (self.flags & 16 == 16).into(),
            (self.flags & 32 == 32).into(),
            (self.flags & 64 == 64).into(),
            (self.flags & 128 == 128).into(),
        ]
    }

    fn submessage_length(&self) -> u16 {
        self.submessage_length
    }
}
