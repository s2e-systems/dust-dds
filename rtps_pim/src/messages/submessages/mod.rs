pub mod submessage_elements;

pub mod ack_nack_submessage;
// pub mod data_frag_submessage;
// pub mod data_submessage;
// pub mod gap_submessage;
// pub mod heartbeat_frag_submessage;
// pub mod heartbeat_submessage;
// pub mod info_destination_submessage;
// pub mod info_reply_submessage;
// pub mod info_source_submessage;
// pub mod info_timestamp_submessage;
// pub mod nack_frag_submessage;
// pub mod pad;

use crate::messages::types;
// pub use ack_nack_submessage::AckNack;
// pub use data_submessage::Data;
// pub use gap_submessage::Gap;
// pub use heartbeat_submessage::Heartbeat;
// pub use info_timestamp_submessage::InfoTimestamp;

pub struct SubmessageHeader<
    SubmessageKind: types::SubmessageKind,
    SubmessageFlag: types::SubmessageFlag,
> {
    pub submessage_id: SubmessageKind,
    pub flags: [SubmessageFlag; 8],
    pub submessage_length: u16,
}
pub trait Submessage {
    fn is_valid(&self) -> bool;
}
