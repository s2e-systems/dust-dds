pub mod submessage_elements;

pub mod ack_nack_submessage;
pub mod data_frag_submessage;
pub mod data_submessage;
pub mod gap_submessage;
pub mod heartbeat_frag_submessage;
pub mod heartbeat_submessage;
pub mod info_destination_submessage;
pub mod info_reply_submessage;
pub mod info_source_submessage;
pub mod info_timestamp_submessage;
pub mod nack_frag_submessage;

use super::types::{SubmessageKind, SubmessageFlag};
pub use ack_nack_submessage::AckNack;
pub use data_submessage::Data;
pub use gap_submessage::Gap;
pub use heartbeat_submessage::Heartbeat;
pub use info_timestamp_submessage::InfoTs;


#[derive(PartialEq, Debug)]
pub struct SubmessageHeader {
    submessage_id: SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: submessage_elements::UShort,
}

impl SubmessageHeader {
    pub fn new(submessage_id: SubmessageKind, flags: [SubmessageFlag; 8], submessage_length: u16) -> Self {
        Self {
            submessage_id, 
            flags,
            submessage_length,
        }
    }

    pub fn submessage_id(&self) -> SubmessageKind {
        self.submessage_id
    }

    pub fn flags(&self) -> &[SubmessageFlag; 8] {
        &self.flags
    }
    pub fn submessage_length(&self) -> submessage_elements::UShort {
        self.submessage_length
    }
}

/// 8.3.7 RTPS Submessages
/// The RTPS protocol version 2.4 defines several kinds of Submessages. 
/// They are categorized into two groups: Entity- Submessages and Interpreter-Submessages.
/// Entity Submessages target an RTPS Entity.
/// Interpreter Submessages modify the RTPS Receiver state and provide context that helps process subsequent Entity Submessages.

#[derive(Debug, PartialEq)]
pub enum RtpsSubmessage {
    AckNack(AckNack),
    Data(Data),
    // DataFrag(DataFrag),
    Gap(Gap),
    Heartbeat(Heartbeat),
    // HeartbeatFrag(HeartbeatFrag),
    // InfoDst(InfoDst),
    // InfoReply(InfoReply),
    // InfoSrc(InfoSrc),
    InfoTs(InfoTs),
    // NackFrag(NackFrag),
}

impl RtpsSubmessage {
    pub fn is_entity_submessage(&self) -> bool {
        match self {
            RtpsSubmessage::Data(_) | 
            // RtpsSubmessage::DataFrag(_) |
            RtpsSubmessage::Heartbeat(_) |
            // RtpsSubmessage::HeartbeatFrag(_) |
            RtpsSubmessage::Gap(_) |
            RtpsSubmessage::AckNack(_) //|
            /*RtpsSubmessage::NackFrag(_)*/ => true,
            _ => false,
        }
    }

    pub fn is_interpreter_submessage(&self) -> bool {
        !self.is_entity_submessage()
    }
}

pub trait Submessage 
{
    fn submessage_header(&self, octets_to_next_header: u16 /* Transport dependent */) -> SubmessageHeader;

    fn is_valid(&self) -> bool;
}