use std::io::{Error, Read, Write};

use byteorder::ByteOrder;
use rtps_pim::messages::{overall_structure::RtpsMessageHeader, types::ProtocolId};

use crate::mapping_traits::{MappingRead, MappingReadByteOrdered, MappingWrite};

impl<const N: usize> MappingWrite for [u8; N] {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        writer.write_all(self)
    }
}
impl<'de, const N: usize> MappingRead<'de> for [u8; N] {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let mut value = [0; N];
        buf.read_exact(value.as_mut())?;
        Ok(value)
    }
}

impl MappingWrite for RtpsMessageHeader {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        match self.protocol {
            ProtocolId::PROTOCOL_RTPS => b"RTPS".mapping_write(&mut writer)?,
        }
        self.version.mapping_write(&mut writer)?;
        self.vendor_id.mapping_write(&mut writer)?;
        self.guid_prefix.mapping_write(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for RtpsMessageHeader {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let protocol: [u8; 4] = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let protocol = if &protocol == b"RTPS" {
            ProtocolId::PROTOCOL_RTPS
        } else {
            return Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Protocol not valid",
            ));
        };
        Ok(Self {
            protocol,
            version: MappingRead::mapping_read(buf)?,
            vendor_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            guid_prefix: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

impl<'de> MappingRead<'de> for RtpsMessageHeader {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let protocol: [u8; 4] = MappingRead::mapping_read(buf)?;
        let protocol = if &protocol == b"RTPS" {
            ProtocolId::PROTOCOL_RTPS
        } else {
            return Result::Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Protocol not valid",
            ));
        };
        Ok(Self {
            protocol,
            version: MappingRead::mapping_read(buf)?,
            vendor_id: MappingRead::mapping_read(buf)?,
            guid_prefix: MappingRead::mapping_read(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rtps_pim::messages::submessage_elements::{
        GuidPrefixSubmessageElement, ProtocolVersionSubmessageElement, VendorIdSubmessageElement,
    };

    use super::*;
    use crate::mapping_traits::{from_bytes, to_bytes};

    #[test]
    fn serialize_rtps_header() {
        let value = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
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
            version: ProtocolVersionSubmessageElement { value: [2, 3] },
            vendor_id: VendorIdSubmessageElement { value: [9, 8] },
            guid_prefix: GuidPrefixSubmessageElement { value: [3; 12] },
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
