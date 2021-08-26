use std::io::Write;

use byteorder::{ByteOrder, LittleEndian};
use rust_rtps_pim::messages::{types::ProtocolId, RtpsMessageHeader};

use crate::{deserialize::{self, Deserialize}, serialize::{self, Mapping, Serialize}};

// impl Serialize for RtpsMessageHeader {
//     fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
//         match self.protocol {
//             ProtocolId::PROTOCOL_RTPS => b"RTPS".serialize::<_, B>(&mut writer)?,
//         }
//         self.version.serialize::<_, B>(&mut writer)?;
//         self.vendor_id.serialize::<_, B>(&mut writer)?;
//         self.guid_prefix.serialize::<_, B>(&mut writer)
//     }
// }

impl Mapping for RtpsMessageHeader {
    fn mapping<W: Write>(&self, mut writer: W) -> serialize::Result {
        match self.protocol {
            ProtocolId::PROTOCOL_RTPS => b"RTPS".mapping(&mut writer)?,
        }
        self.version.mapping(&mut writer)?;
        self.vendor_id.mapping(&mut writer)?;
        self.guid_prefix.mapping(&mut writer)
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
#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::ProtocolVersion;

    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::{to_bytes, to_bytes_le};

    #[test]
    fn serialize_rtps_header() {
        let value = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion { major: 2, minor: 3 },
            vendor_id: [9, 8],
            guid_prefix: [3; 12],
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
            guid_prefix: [3; 12],
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
