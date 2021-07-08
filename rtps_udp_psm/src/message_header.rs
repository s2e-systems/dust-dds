use rust_rtps_pim::structure::types::{ProtocolVersion, VendorId};

use crate::submessage_elements::{GuidPrefixUdp, ProtocolVersionUdp, VendorIdUdp};

pub type ProtocolId = [u8; 4];

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RTPSMessageHeaderUdp {
    pub(crate) protocol: ProtocolId,
    pub(crate) version: ProtocolVersionUdp,
    pub(crate) vendor_id: VendorIdUdp,
    pub(crate) guid_prefix: GuidPrefixUdp,
}

impl<'a> rust_rtps_pim::messages::RtpsMessageHeaderType for RTPSMessageHeaderUdp {
    type ProtocolIdType = ProtocolId;
    type ProtocolVersionType = ProtocolVersion;
    type VendorIdType = VendorId;
    type GuidPrefixType = rust_rtps_pim::structure::types::GuidPrefix;
    const PROTOCOL_RTPS: ProtocolId = [b'R', b'T', b'P', b'S'];

    fn protocol(&self) -> ProtocolId {
        self.protocol
    }

    fn version(&self) -> Self::ProtocolVersionType {
        // &self.version
        todo!()
    }

    fn vendor_id(&self) -> Self::VendorIdType {
        // &self.vendor_id
        todo!()
    }

    fn guid_prefix(&self) -> Self::GuidPrefixType {
        //&self.guid_prefix
        todo!()
    }

    fn new(
        version: &Self::ProtocolVersionType,
        vendor_id: &Self::VendorIdType,
        guid_prefix: &Self::GuidPrefixType,
    ) -> Self {
        Self {
            protocol: Self::PROTOCOL_RTPS,
            version: ProtocolVersionUdp { major: 2, minor: 4 },
            vendor_id: VendorIdUdp([1, 1]),
            guid_prefix: GuidPrefixUdp([1; 12]),
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
}
