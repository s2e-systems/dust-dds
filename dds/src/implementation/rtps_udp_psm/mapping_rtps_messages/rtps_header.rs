use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{overall_structure::RtpsMessageHeader, types::ProtocolId},
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for RtpsMessageHeader {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        match self.protocol {
            ProtocolId::PROTOCOL_RTPS => b"RTPS".mapping_write_byte_ordered::<_, B>(&mut writer)?,
        }
        self.version
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.vendor_id
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.guid_prefix
            .mapping_write_byte_ordered::<_, B>(&mut writer)
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
            version: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            vendor_id: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            guid_prefix: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::types::{GuidPrefix, ProtocolVersion, VendorId},
        rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le},
    };

    use super::*;

    #[test]
    fn serialize_rtps_header() {
        let value = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
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
    fn deserialize_rtps_header() {
        let expected = RtpsMessageHeader {
            protocol: ProtocolId::PROTOCOL_RTPS,
            version: ProtocolVersion::new(2, 3),
            vendor_id: VendorId::new([9, 8]),
            guid_prefix: GuidPrefix::new([3; 12]),
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
