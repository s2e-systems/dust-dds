use crate::{RtpsUdpPsm, SubmessageFlag, SubmessageKind, Octet};

pub mod ack_nack;
pub mod data;
pub mod data_frag;
pub mod gap;
pub mod heartbeat;
pub mod heartbeat_frag;
pub mod info_destination;
pub mod info_reply;
pub mod info_source;
pub mod info_timestamp;
pub mod nack_frag;
pub mod pad;




#[derive(Debug, serde::Serialize)]
pub struct SubmessageHeader {
    submessage_id: Octet,
    flags: Octet,
    submessage_length: u16,
}

impl PartialEq for SubmessageHeader {
    fn eq(&self, other: &Self) -> bool {
        self.flags == other.flags &&
        self.submessage_length == other.submessage_length
    }
}

impl rust_rtps_pim::messages::RtpsSubmessageHeaderType<RtpsUdpPsm> for SubmessageHeader {
    fn submessage_id(&self) -> SubmessageKind {
        self.submessage_id.into()
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
        let flags: Octet = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
        let submessage_length: u16 = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;
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
        D: serde::Deserializer<'de> {
        const FIELDS: &'static [&'static str] = &[
            "submessage_id",
            "flags",
            "submessage_length",
        ];
        deserializer.deserialize_struct("SubmessageHeader", FIELDS, SubmessageHeaderVisitor)
    }
}