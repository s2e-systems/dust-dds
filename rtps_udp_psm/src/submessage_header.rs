use std::convert::TryFrom;

use rust_rtps_pim::messages::types::SubmessageKind;

use crate::submessage_elements::Octet;

impl From<SubmessageKind> for Octet {
    fn from(value: SubmessageKind) -> Self {
        match value {
            SubmessageKind::DATA => Octet(0x15),
            SubmessageKind::GAP => Octet(0x08),
            SubmessageKind::HEARTBEAT => Octet(0x07),
            SubmessageKind::ACKNACK => Octet(0x06),
            SubmessageKind::PAD => Octet(0x01),
            SubmessageKind::INFO_TS => Octet(0x09),
            SubmessageKind::INFO_REPLY => Octet(0x0f),
            SubmessageKind::INFO_DST => Octet(0x0e),
            SubmessageKind::INFO_SRC => Octet(0x0c),
            SubmessageKind::DATA_FRAG => Octet(0x16),
            SubmessageKind::NACK_FRAG => Octet(0x12),
            SubmessageKind::HEARTBEAT_FRAG => Octet(0x13),
        }
    }
}

impl TryFrom<Octet> for SubmessageKind {
    type Error = std::io::Error;

    fn try_from(value: Octet) -> Result<Self, Self::Error> {
        let submessage_kind = match value {
            Octet(0x15) => SubmessageKind::DATA,
            Octet(0x08) => SubmessageKind::GAP,
            Octet(0x07) => SubmessageKind::HEARTBEAT,
            Octet(0x06) => SubmessageKind::ACKNACK,
            Octet(0x01) => SubmessageKind::PAD,
            Octet(0x09) => SubmessageKind::INFO_TS,
            Octet(0x0f) => SubmessageKind::INFO_REPLY,
            Octet(0x0e) => SubmessageKind::INFO_DST,
            Octet(0x0c) => SubmessageKind::INFO_SRC,
            Octet(0x16) => SubmessageKind::DATA_FRAG,
            Octet(0x12) => SubmessageKind::NACK_FRAG,
            Octet(0x13) => SubmessageKind::HEARTBEAT_FRAG,
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "")),
        };
        Ok(submessage_kind)
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
