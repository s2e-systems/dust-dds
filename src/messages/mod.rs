use self::types::{SubmessageFlag, SubmessageKind};
use serdes::{RtpsSerdesResult, SizeSerializer, };

mod ack_nack_submessage;
mod data_frag_submessage;
mod data_submessage;
mod gap_submessage;
mod heartbeat_frag_submessage;
mod heartbeat_submessage;
mod info_destination_submessage;
mod info_reply_submessage;
mod info_source_submessage;
mod info_timestamp_submessage;
mod nack_frag_submessage;

mod message;
mod submessage;
mod serdes;
mod submessage_elements;
pub mod types;
pub mod receiver;

pub use ack_nack_submessage::AckNack;
// pub use data_frag_submessage::DataFrag;
pub use data_submessage::Data;
pub use data_submessage::Payload;
pub use gap_submessage::Gap;
// pub use heartbeat_frag_submessage::HeartbeatFrag;
pub use heartbeat_submessage::Heartbeat;
// pub use info_destination_submessage::InfoDst;
// pub use info_reply_submessage::InfoReply;
// pub use info_source_submessage::InfoSrc;
pub use info_timestamp_submessage::InfoTs;
// pub use nack_frag_submessage::NackFrag;

pub use message::RtpsMessage;
pub use submessage::RtpsSubmessage;
pub use serdes::Endianness;
pub use submessage_elements::ParameterList;

pub const RTPS_MAJOR_VERSION: u8 = 2;
pub const RTPS_MINOR_VERSION: u8 = 4;

pub trait Pid {
    fn pid() -> types::ParameterId;
}

pub trait UdpPsmMapping
    where Self: std::marker::Sized {
    
    fn compose(&self, writer: &mut impl std::io::Write) -> RtpsSerdesResult<()>;
    
    fn octets(&self) -> usize {
        let mut size_serializer = SizeSerializer::new();
        self.compose(&mut size_serializer).unwrap(); // Should panic on failure
        size_serializer.get_size()
    }

    fn parse(bytes: &[u8]) -> RtpsSerdesResult<Self>;
}
