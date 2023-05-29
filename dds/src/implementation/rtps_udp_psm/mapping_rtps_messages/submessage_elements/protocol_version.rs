use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::ProtocolVersion,
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl NumberOfBytes for ProtocolVersion {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl MappingWriteByteOrdered for ProtocolVersion {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.major()
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.minor().mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ProtocolVersion {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let major: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let minor: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Self::new(major, minor))
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::{
        rtps::types::ProtocolVersion, rtps_udp_psm::mapping_traits::to_bytes_le,
    };

    #[test]
    fn serialize_protocol_version() {
        let data = ProtocolVersion::new(2, 3);
        assert_eq!(to_bytes_le(&data).unwrap(), vec![2, 3]);
    }
}
