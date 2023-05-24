use std::{
    io::{Error, Write},
    iter::FromIterator,
};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::{messages::submessage_elements::SequenceNumberSet, types::SequenceNumber},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl NumberOfBytes for SequenceNumberSet {
    fn number_of_bytes(&self) -> usize {
        let num_bits = if let Some(max) = self.set.iter().max() {
            <i64>::from(*max) - <i64>::from(self.base) + 1
        } else {
            0
        } as usize;
        let number_of_bitmap_elements = (num_bits + 31) / 32; // aka "M"
        12 /*bitmapBase + numBits */ + 4 * number_of_bitmap_elements /* bitmap[0] .. bitmap[M-1] */
    }
}

impl MappingWriteByteOrdered for SequenceNumberSet {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let mut bitmap = [0; 8];
        let mut num_bits = 0;
        for sequence_number in &self.set {
            let delta_n = <i64>::from(*sequence_number - self.base) as u32;
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
            if delta_n + 1 > num_bits {
                num_bits = delta_n + 1;
            }
        }
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard refered to as "M"

        let high = (<i64>::from(self.base) >> 32) as i32;
        let low = <i64>::from(self.base) as i32;
        high.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        low.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        num_bits.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        for bitmap_element in &bitmap[..number_of_bitmap_elements] {
            bitmap_element.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}

impl<'de> MappingReadByteOrdered<'de> for SequenceNumberSet {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let high: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let low: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let base = ((high as i64) << 32) + low as i64;

        let num_bits: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard refered to as "M"
        let mut bitmap = [0; 8];
        for bitmap_i in bitmap.iter_mut().take(number_of_bitmap_elements) {
            *bitmap_i = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        }

        let mut set = Vec::with_capacity(256);
        for delta_n in 0..num_bits as usize {
            if (bitmap[delta_n / 32] & (1 << (31 - delta_n % 32))) == (1 << (31 - delta_n % 32)) {
                set.push(SequenceNumber::new(base + delta_n as i64));
            }
        }
        Ok(Self {
            base: SequenceNumber::new(base),
            set: Vec::from_iter(set.into_iter()),
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le};

    use super::*;

    #[test]
    fn serialize_sequence_number_max_gap() {
        let sequence_number_set = SequenceNumberSet {
            base: SequenceNumber::new(2),
            set: vec![SequenceNumber::new(2), SequenceNumber::new(257)],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&sequence_number_set).unwrap(), vec![
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
    }

    #[test]
    fn deserialize_sequence_number_set_max_gap() {
        let expected = SequenceNumberSet {
            base: SequenceNumber::new(2),
            set: vec![SequenceNumber::new(2), SequenceNumber::new(257)],
        };
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            0, 0, 0, 0, // bitmapBase: high (long)
            2, 0, 0, 0, // bitmapBase: low (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn number_of_bytes_max_numbers() {
        let sequence_number_set = SequenceNumberSet {
            base: SequenceNumber::new(2),
            set: vec![SequenceNumber::new(2), SequenceNumber::new(257)],
        };
        assert_eq!(sequence_number_set.number_of_bytes(), 44);
    }

    #[test]
    fn number_of_bytes_empty() {
        let sequence_number_set = SequenceNumberSet {
            base: SequenceNumber::new(2),
            set: vec![],
        };
        assert_eq!(sequence_number_set.number_of_bytes(), 12);
    }

    #[test]
    fn number_of_bytes_one() {
        let sequence_number_set = SequenceNumberSet {
            base: SequenceNumber::new(2),
            set: vec![SequenceNumber::new(257)],
        };
        assert_eq!(sequence_number_set.number_of_bytes(), 44);
    }
}
