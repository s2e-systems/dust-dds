use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::types::FragmentNumber,
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for FragmentNumber {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <u32>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for FragmentNumber {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(FragmentNumber::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps_udp_psm::mapping_traits::to_bytes_le;

    #[test]
    fn serialize_fragment_number() {
        let data = FragmentNumber::new(7);
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                7, 0, 0, 0, // (unsigned long)
            ]
        );
    }
}
