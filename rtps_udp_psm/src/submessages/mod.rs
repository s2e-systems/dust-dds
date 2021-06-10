use crate::{RtpsUdpPsm, SubmessageFlag, SubmessageKind, Octet};

pub mod ack_nack;
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




#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SubmessageHeader {
    submessage_id: Octet,
    flags: Octet,
    submessage_length: u16,
}

impl rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> for SubmessageHeader {
    fn submessage_id(&self) -> SubmessageKind {
        self.submessage_id.into()
    }

    fn flags(&self) -> [SubmessageFlag; 8] {
       self.flags.into()
    }

    fn submessage_length(&self) -> u16 {
        self.submessage_length
    }
}
