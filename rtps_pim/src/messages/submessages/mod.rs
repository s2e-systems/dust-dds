pub mod submessage_elements;

pub mod ack_nack_submessage;
// pub mod data_frag_submessage;
pub mod data_submessage;
// pub mod gap_submessage;
// pub mod heartbeat_frag_submessage;
// pub mod heartbeat_submessage;
// pub mod info_destination_submessage;
// pub mod info_reply_submessage;
// pub mod info_source_submessage;
// pub mod info_timestamp_submessage;
// pub mod nack_frag_submessage;
// pub mod pad;

use super::types::{SubmessageFlag, SubmessageKind};
// pub use ack_nack_submessage::AckNack;
// pub use data_submessage::Data;
// pub use gap_submessage::Gap;
// pub use heartbeat_submessage::Heartbeat;
// pub use info_timestamp_submessage::InfoTimestamp;

pub trait SubmessageHeader : Copy {
    type SubmessageKind: SubmessageKind;
    type SubmessageFlag : SubmessageFlag;

    fn submessage_id(&self) -> Self::SubmessageKind;
    fn flags(&self) -> [Self::SubmessageFlag; 8];
    fn submessage_length(&self) -> u16;
}
pub trait Submessage {
    type SubmessageHeader: SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader;
}
