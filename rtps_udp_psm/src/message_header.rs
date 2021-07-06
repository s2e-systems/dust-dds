use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{ProtocolVersion, VendorId},
};

pub type ProtocolId = [u8; 4];

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RTPSMessageHeader {
    pub(crate) protocol: ProtocolId,
    pub(crate) version: ProtocolVersion,
    pub(crate) vendor_id: VendorId,
    pub(crate) guid_prefix: rust_rtps_pim::structure::types::GuidPrefix,
}

impl<'a> rust_rtps_pim::messages::RtpsMessageHeaderType<RtpsUdpPsm> for RTPSMessageHeader {
    fn protocol(&self) -> &ProtocolId {
        &self.protocol
    }

    fn version(&self) -> &rust_rtps_pim::structure::types::ProtocolVersion {
        // &self.version
        todo!()
    }

    fn vendor_id(&self) -> &rust_rtps_pim::structure::types::VendorId {
        // &self.vendor_id
        todo!()
    }

    fn guid_prefix(&self) -> &rust_rtps_pim::structure::types::GuidPrefix {
        &self.guid_prefix
    }
}



#[cfg(test)]
mod tests {
    use rust_serde_cdr::{deserializer::RtpsMessageDeserializer, serializer::RtpsMessageSerializer};

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
        let value = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
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
        let expected = RTPSMessageHeader {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: VendorId([9, 8]),
            guid_prefix: [3; 12],
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
}
