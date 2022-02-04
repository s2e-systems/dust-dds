use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::structure::types::SequenceNumber;

use crate::{
    mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
    messages::submessage_elements::SequenceNumberSubmessageElementPsm,
};

impl MappingWriteByteOrdered for SequenceNumber {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let high = (self >> 32) as i32;
        let low = *self as i32;
        high.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        low.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for SequenceNumber {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let high: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let low: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(((high as i64) << 32) + low as i64)
    }
}

impl MappingWriteByteOrdered for SequenceNumberSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for SequenceNumberSubmessageElementPsm {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping_traits::{from_bytes_le, to_bytes_le};

    #[test]
    fn serialize_sequence_number() {
        let data = SequenceNumberSubmessageElementPsm { value: 7 };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ]
        );
    }

    #[test]
    fn deserialize_sequence_number() {
        let expected = SequenceNumberSubmessageElementPsm { value: 7 };
        assert_eq!(
            expected,
            from_bytes_le(&[
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ])
            .unwrap()
        );
    }
}
