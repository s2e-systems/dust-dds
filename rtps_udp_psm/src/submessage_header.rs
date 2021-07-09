use rust_rtps_pim::messages::types::{SubmessageFlag, SubmessageKind};

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

#[derive(Debug, serde::Serialize)]
pub struct SubmessageHeader {
    pub(crate) submessage_id: Octet,
    pub(crate) flags: Octet,
    pub(crate) submessage_length: u16,
}

impl PartialEq for SubmessageHeader {
    fn eq(&self, other: &Self) -> bool {
        self.flags == other.flags && self.submessage_length == other.submessage_length
    }
}

impl<'a> rust_rtps_pim::messages::RtpsSubmessageHeaderType for SubmessageHeader {
    fn submessage_id(&self) -> SubmessageKind {
        // self.submessage_id.into()
        todo!()
    }

    fn flags(&self) -> [SubmessageFlag; 8] {
        self.flags.into()
    }

    fn submessage_length(&self) -> u16 {
        self.submessage_length
    }
}

struct SubmessageHeaderVisitor;

impl<'de> serde::de::Visitor<'de> for SubmessageHeaderVisitor {
    type Value = SubmessageHeader;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("SubmessageHeaderVisitor")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let flags: Octet = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let submessage_length: u16 = seq
            .next_element()?
            .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
        Ok(SubmessageHeader {
            submessage_id: 0.into(),
            flags,
            submessage_length,
        })
    }
}

impl<'de> serde::Deserialize<'de> for SubmessageHeader {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &'static [&'static str] = &["submessage_id", "flags", "submessage_length"];
        deserializer.deserialize_struct("SubmessageHeader", FIELDS, SubmessageHeaderVisitor)
    }
}
