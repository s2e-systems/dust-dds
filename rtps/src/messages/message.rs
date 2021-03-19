use serde::{ser::SerializeStruct, Serialize};

use crate::types::{GuidPrefix, ProtocolVersion, VendorId};

use super::{
    submessages::Submessage,
    types::{constants::PROTOCOL_RTPS, ProtocolId},
};

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Header {
    protocol: ProtocolId,
    version: ProtocolVersion,
    vendor_id: VendorId,
    guid_prefix: GuidPrefix,
}

impl Header {
    pub fn new(version: ProtocolVersion, vendor_id: VendorId, guid_prefix: GuidPrefix) -> Self {
        Self {
            protocol: PROTOCOL_RTPS,
            version,
            vendor_id,
            guid_prefix,
        }
    }

    pub fn protocol(&self) -> ProtocolId {
        self.protocol
    }

    pub fn version(&self) -> ProtocolVersion {
        self.version
    }

    pub fn vendor_id(&self) -> VendorId {
        self.vendor_id
    }

    pub fn guid_prefix(&self) -> GuidPrefix {
        self.guid_prefix
    }
}

pub struct RtpsMessage<'a> {
    header: Header,
    submessages: &'a [&'a dyn Submessage],
}

impl<'a> RtpsMessage<'a> {
    pub fn new(
        version: ProtocolVersion,
        vendor_id: VendorId,
        guid_prefix: GuidPrefix,
        submessages: &'a [&'a dyn Submessage],
    ) -> Self {
        if submessages.is_empty() {
            panic!("At least one submessage is required");
        };

        RtpsMessage {
            header: Header::new(version, vendor_id, guid_prefix),
            submessages,
        }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn submessages(&self) -> &[&dyn Submessage] {
        &self.submessages
    }
}

impl<'a> serde::Serialize for RtpsMessage<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("RtpsMessage", 2)?;
        state.serialize_field("header", &self.header)?;
        for submessage in self.submessages{
            state.serialize_field("submessage", submessage)?;
        }
        state.end()
    }
}

impl<'a> serde::Serialize for &'a dyn Submessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // ATTENTION: The serialize must be called for the dereferenced self
        // otherwise this function is called in an infinite loop and a stack
        // overflow error pops-up
        erased_serde::serialize(*self,serializer)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::constants::{PROTOCOL_VERSION_2_4, VENDOR_ID};
    use serde_test::{assert_ser_tokens, Token};

    use super::*;

    struct MockSubmessage;

    impl Submessage for MockSubmessage {
        fn submessage_header(
            &self,
            _octets_to_next_header: u16, /* Transport dependent */
        ) -> crate::messages::submessages::SubmessageHeader {
            todo!()
        }

        fn is_valid(&self) -> bool {
            todo!()
        }
    }

    impl serde::Serialize for MockSubmessage {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_u8(5)
        }
    }

    #[test]
    fn serialize_header() {
        let rtps_header = Header::new(PROTOCOL_VERSION_2_4, [99, 99], [1; 12]);
        assert_ser_tokens(
            &rtps_header,

            &[
                Token::Struct {
                    name: "Header",
                    len: 4,
                },
                Token::Str("protocol"),
                Token::Tuple { len: 4 },
                Token::U8(b'R'),
                Token::U8(b'T'),
                Token::U8(b'P'),
                Token::U8(b'S'),
                Token::TupleEnd,
                Token::Str("version"),
                Token::Struct {
                    name: "ProtocolVersion",
                    len: 2,
                },
                Token::Str("major"),
                Token::U8(2),
                Token::Str("minor"),
                Token::U8(4),
                Token::StructEnd,
                Token::Str("vendor_id"),
                Token::Tuple { len: 2 },
                Token::U8(99),
                Token::U8(99),
                Token::TupleEnd,
                Token::Str("guid_prefix"),
                Token::Tuple { len: 12 },
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::TupleEnd,
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn serialize_message() {
        let rtps_message =
            RtpsMessage::new(PROTOCOL_VERSION_2_4, VENDOR_ID, [1; 12], &[&MockSubmessage, &MockSubmessage]);

        assert_ser_tokens(
            &rtps_message,
            &[
                Token::Struct {
                    name: "RtpsMessage",
                    len: 2,
                },
                Token::Str("header"),
                Token::Struct {
                    name: "Header",
                    len: 4,
                },
                Token::Str("protocol"),
                Token::Tuple { len: 4 },
                Token::U8(b'R'),
                Token::U8(b'T'),
                Token::U8(b'P'),
                Token::U8(b'S'),
                Token::TupleEnd,
                Token::Str("version"),
                Token::Struct {
                    name: "ProtocolVersion",
                    len: 2,
                },
                Token::Str("major"),
                Token::U8(2),
                Token::Str("minor"),
                Token::U8(4),
                Token::StructEnd,
                Token::Str("vendor_id"),
                Token::Tuple { len: 2 },
                Token::U8(99),
                Token::U8(99),
                Token::TupleEnd,
                Token::Str("guid_prefix"),
                Token::Tuple { len: 12 },
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::U8(1),
                Token::TupleEnd,
                Token::StructEnd,
                Token::Str("submessage"),
                Token::U8(5),
                Token::Str("submessage"),
                Token::U8(5),
                Token::StructEnd,
            ],
        );
    }
}
