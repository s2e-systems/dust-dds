use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::{overall_structure::RtpsMessageHeader, types::ProtocolId};

use crate::{
    deserialize::{self, Deserialize, MappingRead},
    serialize::{self, MappingWrite},
};

impl MappingWrite for RtpsMessageHeader {
    fn write<W: Write>(&self, mut writer: W) -> serialize::Result {
        match self.protocol {
            ProtocolId::PROTOCOL_RTPS => b"RTPS".write(&mut writer)?,
        }
        self.version.write(&mut writer)?;
        self.vendor_id.write(&mut writer)?;
        self.guid_prefix.write(&mut writer)
    }
}

impl<'de> Deserialize<'de> for RtpsMessageHeader {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let protocol: [u8; 4] = Deserialize::deserialize::<B>(buf)?;
        let protocol = if &protocol == b"RTPS" {
            ProtocolId::PROTOCOL_RTPS
        } else {
            return deserialize::Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Protocol not valid",
            ));
        };
        Ok(Self {
            protocol,
            version: Deserialize::deserialize::<B>(buf)?,
            vendor_id: Deserialize::deserialize::<B>(buf)?,
            guid_prefix: Deserialize::deserialize::<B>(buf)?,
        })
    }
}

impl<'de> MappingRead<'de> for RtpsMessageHeader {
    fn read(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let protocol: [u8; 4] = MappingRead::read(buf)?;
        let protocol = if &protocol == b"RTPS" {
            ProtocolId::PROTOCOL_RTPS
        } else {
            return deserialize::Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Protocol not valid",
            ));
        };
        Ok(Self {
            protocol,
            version: MappingRead::read(buf)?,
            vendor_id: MappingRead::read(buf)?,
            guid_prefix: MappingRead::read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::{GuidPrefix, ProtocolVersion};

    use super::*;
    use crate::deserialize::from_bytes;
    use crate::serialize::to_bytes;

    #[test]
    fn serialize_rtps_header() {
        let value = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: GuidPrefix([3; 12]),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes(&value).unwrap(), vec![
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]);
    }

    #[test]
    fn deserialize_rtps_header() {
        let expected = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: GuidPrefix([3; 12]),
        };
        #[rustfmt::skip]
        let result = from_bytes(&[
            b'R', b'T', b'P', b'S', // Protocol
            2, 3, 9, 8, // ProtocolVersion | VendorId
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
            3, 3, 3, 3, // GuidPrefix
        ]).unwrap();
        assert_eq!(expected, result);
    }
}
