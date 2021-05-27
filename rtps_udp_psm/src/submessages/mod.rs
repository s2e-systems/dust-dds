use crate::{RtpsUdpPsm, SubmessageFlag, SubmessageKind};

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

pub struct SubmessageHeader {}

impl rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> for SubmessageHeader {
    fn submessage_id(&self) -> SubmessageKind {
        todo!()
    }

    fn flags(&self) -> [SubmessageFlag; 8] {
        todo!()
    }

    fn submessage_length(&self) -> u16 {
        todo!()
    }
}
