pub mod submessage_elements;

pub mod ack_nack_submessage;
pub mod data_frag_submessage;
pub mod data_submessage;
pub mod gap_submessage;
// pub mod heartbeat_frag_submessage;
// pub mod heartbeat_submessage;
// pub mod info_destination_submessage;
// pub mod info_reply_submessage;
// pub mod info_source_submessage;
// pub mod info_timestamp_submessage;
// pub mod nack_frag_submessage;
// pub mod pad;

use crate::messages;
// pub use ack_nack_submessage::AckNack;
// pub use data_submessage::Data;
// pub use gap_submessage::Gap;
// pub use heartbeat_submessage::Heartbeat;
// pub use info_timestamp_submessage::InfoTimestamp;

pub struct SubmessageHeader<
    SubmessageKind: messages::types::SubmessageKind,
    SubmessageFlag: messages::types::SubmessageFlag,
> {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}

pub trait Submessage {
    type SubmessageKind: messages::types::SubmessageKind;
    type SubmessageFlag: messages::types::SubmessageFlag;
    fn submessage_header(&self) -> SubmessageHeader<Self::SubmessageKind, Self::SubmessageFlag>;
}
