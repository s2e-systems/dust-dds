use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::GuidPrefix,
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for GuidPrefix {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <[u8; 12]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GuidPrefix {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}

impl NumberOfBytes for GuidPrefix {
    fn number_of_bytes(&self) -> usize {
        12
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps_udp_psm::mapping_traits::to_bytes_le;

    use super::*;

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
