use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::RtpsMessageHeader, structure::types::ProtocolVersion};

use crate::submessage_elements::{GuidPrefixUdp, ProtocolVersionUdp, VendorIdUdp};

pub type ProtocolIdUdp = [u8; 4];

#[derive(Debug, PartialEq)]
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

impl crate::serialize::Serialize for RTPSMessageHeaderUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> crate::serialize::Result {
        self.protocol.serialize::<_, B>(&mut writer)?;
        self.version.serialize::<_, B>(&mut writer)?;
        self.vendor_id.serialize::<_, B>(&mut writer)?;
        self.guid_prefix.serialize::<_, B>(&mut writer)
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for RTPSMessageHeaderUdp {
    fn deserialize<B>(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self>
    where
        B: ByteOrder,
    {
        let protocol = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let version = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let vendor_id = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        let guid_prefix = crate::deserialize::Deserialize::deserialize::<B>(buf)?;
        Ok(Self {
            protocol,
            version,
            vendor_id,
            guid_prefix,
        })
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
    use super::*;
    use crate::{deserialize::from_bytes_le, serialize::to_bytes_le};

    #[test]
    fn serialize_rtps_message_header() {
        let value = RTPSMessageHeaderUdp {
            protocol: b"RTPS".to_owned(),
            version: ProtocolVersionUdp { major: 2, minor: 3 },
            vendor_id: VendorIdUdp([9, 8]),
            guid_prefix: GuidPrefixUdp([3; 12]),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&value).unwrap(), vec![
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
        let result = from_bytes_le(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
