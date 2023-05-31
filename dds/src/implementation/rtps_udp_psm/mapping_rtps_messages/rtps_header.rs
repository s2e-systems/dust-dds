use crate::implementation::{
    rtps::messages::{overall_structure::RtpsMessageHeader, types::ProtocolId},
    rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

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

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::types::{GuidPrefix, ProtocolVersion, VendorId},
        rtps_udp_psm::mapping_traits::to_bytes_le,
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
}
