use crate::implementation::{
    rtps::types::SequenceNumber, rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteByteOrdered for SequenceNumber {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let high = (<i64>::from(*self) >> 32) as i32;
        let low = <i64>::from(*self) as i32;
        high.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        low.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps_udp_psm::mapping_traits::to_bytes_le;

    #[test]
    fn serialize_sequence_number() {
        let data = SequenceNumber::new(7);
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ]
        );
    }
}
