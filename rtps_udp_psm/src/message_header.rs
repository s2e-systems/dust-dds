use rust_rtps_pim::{messages::RtpsMessageHeader, structure::types::ProtocolVersion};

use crate::submessage_elements::{GuidPrefixUdp, ProtocolVersionUdp, VendorIdUdp};

pub type ProtocolIdUdp = [u8; 4];

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RTPSMessageHeaderUdp {
    pub(crate) protocol: ProtocolIdUdp,
    pub(crate) version: ProtocolVersionUdp,
    pub(crate) vendor_id: VendorIdUdp,
    pub(crate) guid_prefix: GuidPrefixUdp,
}

impl RTPSMessageHeaderUdp {
    pub const fn number_of_bytes(&self) -> usize {
        20
    }
}

impl From<&RTPSMessageHeaderUdp> for RtpsMessageHeader {
    fn from(header: &RTPSMessageHeaderUdp) -> Self {
        Self {
            protocol: rust_rtps_pim::messages::types::ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion {
                major: header.version.major,
                minor: header.version.minor,
            },
            vendor_id: header.vendor_id.0,
            guid_prefix: header.guid_prefix.0,
        }
    }
}

impl From<&RtpsMessageHeader> for RTPSMessageHeaderUdp {
    fn from(header: &RtpsMessageHeader) -> Self {
        Self {
            protocol: [b'R', b'T', b'P', b'S'],
            version: ProtocolVersionUdp {
                major: header.version.major,
                minor: header.version.minor,
            },
            vendor_id: VendorIdUdp(header.vendor_id),
            guid_prefix: GuidPrefixUdp(header.guid_prefix),
        }
    }
}

#[cfg(test)]
mod tests {
    use rust_serde_cdr::{
        deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer,
    };

    use super::*;

    fn serialize<T: serde::Serialize>(value: T) -> Vec<u8> {
        let mut serializer = RtpsMessageSerializer {
            writer: Vec::<u8>::new(),
        };
        value.serialize(&mut serializer).unwrap();
        serializer.writer
    }

    fn deserialize<'de, T: serde::Deserialize<'de>>(buffer: &'de [u8]) -> T {
        let mut de = RtpsMessageDeserializer { reader: buffer };
        serde::de::Deserialize::deserialize(&mut de).unwrap()
    }

    #[test]
    fn serialize_rtps_message_header() {
        let value = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        #[rustfmt::skip]
        assert_eq!(serialize(value), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn deserialize_rtps_message_header() {
        let expected = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        #[rustfmt::skip]
        let result = deserialize(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_rtps_message_header_json() {
        let value = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        #[rustfmt::skip]
        assert_eq!(serde_json::ser::to_string(&value).unwrap(),
        r#"{"protocol":[82,84,80,83],"version":{"major":2,"minor":3},"vendor_id":[9,8],"guid_prefix":[3,3,3,3,3,3,3,3,3,3,3,3]}"#
        );
    }

    #[test]
    fn deserialize_rtps_message_header_json() {
        let expected = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        let json_str = r#"{"protocol":[82,84,80,83],"version":{"major":2,"minor":3},"vendor_id":[9,8],"guid_prefix":[3,3,3,3,3,3,3,3,3,3,3,3]}"#;
        let result = serde_json::de::from_str(json_str).unwrap();
        assert_eq!(expected, result);
    }
}
