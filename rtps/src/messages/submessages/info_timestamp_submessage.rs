use serde::ser::SerializeStruct;

use super::submessage_elements;
use super::{Submessage, SubmessageFlag, SubmessageHeader};

use crate::messages::types::constants;

#[derive(PartialEq, Debug)]
pub struct InfoTimestamp {
    pub endianness_flag: SubmessageFlag,
    pub invalidate_flag: SubmessageFlag,
    pub timestamp: submessage_elements::Timestamp,
}

impl InfoTimestamp {
    pub fn new(
        endianness_flag: SubmessageFlag,
        invalidate_flag: SubmessageFlag,
        timestamp: submessage_elements::Timestamp,
    ) -> Self {
        Self {
            endianness_flag,
            invalidate_flag,
            timestamp,
        }
    }

    pub const INVALID_TIME_FLAG_MASK: u8 = 0x02;
}

impl Submessage for InfoTimestamp {
    fn submessage_header(&self) -> SubmessageHeader {
        let x = false;
        let e = self.endianness_flag; // Indicates endianness.
        let i = self.invalidate_flag; // Indicates whether subsequent Submessages should be considered as having a timestamp or not.
                                      // X|X|X|X|X|X|I|E
        let flags = [e, i, x, x, x, x, x, x];

        SubmessageHeader::new(constants::SUBMESSAGE_KIND_INFO_TIMESTAMP, flags, 0)
    }

    fn is_valid(&self) -> bool {
        true
    }
}

impl serde::Serialize for InfoTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("InfoTimestamp", 2)?;
        state.serialize_field("header", &self.submessage_header())?;
        if self.invalidate_flag == false {
            state.serialize_field("timestamp", &self.timestamp)?;
        }
        state.end()
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_ser_tokens, Token};

    use crate::messages::types::Time;

    use super::*;

    #[test]
    fn serialize_info_timestamp_invalidate_flag_false() {
        assert_ser_tokens(
            &InfoTimestamp::new(true, false, Time::new(10, 0)),
            &[
                Token::Struct {
                    name: "InfoTimestamp",
                    len: 2,
                },
                Token::Str("header"),
                Token::Struct {
                    name: "SubmessageHeader",
                    len: 3,
                },
                Token::Str("submessage_id"),
                Token::U8(9),
                Token::Str("flags"),
                Token::U8(1),
                Token::Str("submessage_length"),
                Token::U16(0),
                Token::StructEnd,
                Token::Str("timestamp"),
                Token::Struct {
                    name: "Time",
                    len: 2,
                },
                Token::Str("seconds"),
                Token::U32(10),
                Token::Str("fraction"),
                Token::U32(0),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn serialize_info_timestamp_invalidate_flag_true() {
        assert_ser_tokens(
            &InfoTimestamp::new(true, true, Time::new(10, 0)),
            &[
                Token::Struct {
                    name: "InfoTimestamp",
                    len: 2,
                },
                Token::Str("header"),
                Token::Struct {
                    name: "SubmessageHeader",
                    len: 3,
                },
                Token::Str("submessage_id"),
                Token::U8(9),
                Token::Str("flags"),
                Token::U8(3),
                Token::Str("submessage_length"),
                Token::U16(0),
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
