use crate::implementation::{
    rtps::messages::submessage_elements::SequenceNumberSet,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::types::SequenceNumber;

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
