use serdes::{RtpsSerdesResult, SizeSerializer, };

pub mod message;
pub mod submessages;
pub mod serdes;
pub mod types;
pub mod message_receiver;
pub mod message_sender;
pub mod parameter_list;


pub use message::RtpsMessage;
pub use submessages::RtpsSubmessage;
pub use serdes::{Endianness, SubmessageElement};
pub use parameter_list::ParameterList;

pub const RTPS_MAJOR_VERSION: u8 = 2;
pub const RTPS_MINOR_VERSION: u8 = 4;


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
