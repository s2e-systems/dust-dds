use crate::implementation::{
    rtps::types::GuidPrefix,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteByteOrdered for GuidPrefix {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <[u8; 12]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl NumberOfBytes for GuidPrefix {
    fn number_of_bytes(&self) -> usize {
        12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps_udp_psm::mapping_traits::to_bytes_le;

    #[test]
    fn serialize_guid_prefix() {
        let data = GuidPrefix::new([1; 12]);
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&data).unwrap(), vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]);
    }
}
