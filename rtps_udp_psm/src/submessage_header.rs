use rust_rtps_pim::messages::types::SubmessageKind;

use crate::submessage_elements::Octet;


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

impl From<SubmessageKind> for Octet {
    fn from(value: SubmessageKind) -> Self {
        match value {
            SubmessageKind::DATA => Octet(DATA),
            SubmessageKind::GAP => Octet(GAP),
            SubmessageKind::HEARTBEAT => Octet(HEARTBEAT),
            SubmessageKind::ACKNACK => Octet(ACKNACK),
            SubmessageKind::PAD => Octet(PAD),
            SubmessageKind::INFO_TS => Octet(INFO_TS),
            SubmessageKind::INFO_REPLY => Octet(INFO_REPLY),
            SubmessageKind::INFO_DST => Octet(INFO_DST),
            SubmessageKind::INFO_SRC => Octet(INFO_SRC),
            SubmessageKind::DATA_FRAG => Octet(DATA_FRAG),
            SubmessageKind::NACK_FRAG => Octet(NACK_FRAG),
            SubmessageKind::HEARTBEAT_FRAG => Octet(HEARTBEAT_FRAG),
        }
    }
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SubmessageHeaderUdp {
    pub(crate) submessage_id: Octet,
    pub(crate) flags: Octet,
    pub(crate) submessage_length: u16,
}

impl SubmessageHeaderUdp {
    pub const fn number_of_bytes(&self) -> usize {
        4
    }
}
