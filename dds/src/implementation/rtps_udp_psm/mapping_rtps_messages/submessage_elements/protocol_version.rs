use crate::implementation::{
    rtps::types::ProtocolVersion,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

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
