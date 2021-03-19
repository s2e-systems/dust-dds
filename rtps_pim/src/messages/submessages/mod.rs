pub mod submessage_elements;

pub mod ack_nack_submessage;
pub mod data_frag_submessage;
pub mod data_submessage;
pub mod gap_submessage;
pub mod heartbeat_frag_submessage;
pub mod heartbeat_submessage;
pub mod info_destination_submessage;
pub mod info_reply_submessage;
pub mod info_source_submessage;
pub mod info_timestamp_submessage;
pub mod nack_frag_submessage;

use super::types::{SubmessageFlag, SubmessageKind};
pub use ack_nack_submessage::AckNack;
pub use data_submessage::Data;
pub use gap_submessage::Gap;
pub use heartbeat_submessage::Heartbeat;
pub use info_timestamp_submessage::InfoTimestamp;
use serde::ser::SerializeStruct;

#[derive(PartialEq, Debug)]
pub struct SubmessageHeader {
    submessage_id: SubmessageKind,
    flags: [SubmessageFlag; 8],
    submessage_length: submessage_elements::UShort,
}

impl SubmessageHeader {
    pub fn new(
        submessage_id: SubmessageKind,
        flags: [SubmessageFlag; 8],
        submessage_length: u16,
    ) -> Self {
        Self {
            submessage_id,
            flags,
            submessage_length,
        }
    }

    pub fn submessage_id(&self) -> SubmessageKind {
        self.submessage_id
    }

    pub fn flags(&self) -> &[SubmessageFlag; 8] {
        &self.flags
    }
    pub fn submessage_length(&self) -> submessage_elements::UShort {
        self.submessage_length
    }
}

impl serde::Serialize for SubmessageHeader {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("SubmessageHeader", 3)?;
        state.serialize_field("submessage_id", &self.submessage_id)?;
        let mut flags = 0u8;
        for i in 0..8 {
            if self.flags[i] {
                flags |= 0b00000001 << i;
            }
        }
        state.serialize_field("flags", &flags)?;
        state.serialize_field("submessage_length", &self.submessage_length)?;
        state.end()
    }
}
/// 8.3.7 RTPS Submessages
/// The RTPS protocol version 2.4 defines several kinds of Submessages.
/// They are categorized into two groups: Entity- Submessages and Interpreter-Submessages.
/// Entity Submessages target an RTPS Entity.
/// Interpreter Submessages modify the RTPS Receiver state and provide context that helps process subsequent Entity Submessages.

pub trait Submessage: erased_serde::Serialize {
    fn submessage_header(&self) -> SubmessageHeader;

    fn is_valid(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_ser_tokens, Token};

    use super::*;

    #[test]
    fn serialize_submessage_header() {
        let submessage_id = 10;
        let flags = [true, true, true, false, false, false, false, false];
        let submessage_length = 16;
        let submessage_header = SubmessageHeader::new(submessage_id, flags, submessage_length);

        assert_ser_tokens(
            &submessage_header,
            &[
                Token::Struct {
                    name: "SubmessageHeader",
                    len: 3,
                },
                Token::Str("submessage_id"),
                Token::U8(submessage_id),
                Token::Str("flags"),
                Token::U8(7),
                Token::Str("submessage_length"),
                Token::U16(submessage_length),
                Token::StructEnd,
            ],
        )
    }
}
