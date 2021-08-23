use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::types::SubmessageKind;


pub const DATA: u8 = 0x15;
pub const GAP: u8 = 0x08;
pub const HEARTBEAT: u8 = 0x07;
pub const ACKNACK: u8 = 0x06;
pub const PAD: u8 = 0x01;
pub const INFO_TS: u8 = 0x09;
pub const INFO_REPLY: u8 = 0x0f;
pub const INFO_DST: u8 = 0x0e;
pub const INFO_SRC: u8 = 0x0c;
pub const DATA_FRAG: u8 = 0x16;
pub const NACK_FRAG: u8 = 0x12;
pub const HEARTBEAT_FRAG: u8 = 0x13;


pub fn submessage_kind_into_byte(value: SubmessageKind) -> u8 {
    match value {
        SubmessageKind::DATA => DATA,
        SubmessageKind::GAP => GAP,
        SubmessageKind::HEARTBEAT => HEARTBEAT,
        SubmessageKind::ACKNACK => ACKNACK,
        SubmessageKind::PAD => PAD,
        SubmessageKind::INFO_TS => INFO_TS,
        SubmessageKind::INFO_REPLY => INFO_REPLY,
        SubmessageKind::INFO_DST => INFO_DST,
        SubmessageKind::INFO_SRC => INFO_SRC,
        SubmessageKind::DATA_FRAG => DATA_FRAG,
        SubmessageKind::NACK_FRAG => NACK_FRAG,
        SubmessageKind::HEARTBEAT_FRAG => HEARTBEAT_FRAG,
        SubmessageKind::UNKNOWN => panic!("Can not convert unknown into byte"),

    }
}


#[derive(Debug, PartialEq)]
pub struct SubmessageHeaderUdp {
    pub(crate) submessage_id: u8,
    pub(crate) flags: u8,
    pub(crate) submessage_length: u16,
}

impl SubmessageHeaderUdp {
    pub const fn number_of_bytes(&self) -> usize {
        4
    }
}

impl crate::serialize::Serialize for SubmessageHeaderUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.submessage_id.serialize::<_, B>(&mut writer)?;
        self.flags.serialize::<_, B>(&mut writer)?;
        self.submessage_length.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for SubmessageHeaderUdp {
    fn deserialize<B>(buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        let submessage_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let flags = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let submessage_length = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self{ submessage_id, flags, submessage_length})
    }
}