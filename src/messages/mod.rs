pub mod message;
pub mod submessages;
pub mod types;
pub mod message_receiver;
pub mod message_sender;
pub mod parameter_list;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Endianness {
    BigEndian = 0,
    LittleEndian = 1,
}

impl From<bool> for Endianness {
    fn from(value: bool) -> Self {
        match value {
            true => Endianness::LittleEndian,
            false => Endianness::BigEndian,
        }
    }
}

impl From<Endianness> for bool {
    fn from(value: Endianness) -> Self {
        match value {
            Endianness::LittleEndian => true,
            Endianness::BigEndian => false,
        }
    }
}

pub use message::RtpsMessage;
pub use submessages::RtpsSubmessage;
pub use parameter_list::ParameterList;