use std::{
    io::{Error, Write},
    iter::FromIterator,
};

use byteorder::ByteOrder;
use rtps_pim::messages::{
    submessage_elements::FragmentNumberSetSubmessageElement, types::FragmentNumber,
};

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

impl<T> MappingWriteByteOrdered for FragmentNumberSetSubmessageElement<T>
where
    for<'a> &'a T: IntoIterator<Item = &'a FragmentNumber>,
{
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let mut bitmap = [0; 8];
        let mut num_bits = 0;
        for fragment_number in &self.set {
            let delta_n = (fragment_number.0 - self.base.0) as u32;
            let bitmap_num = delta_n / 32;
            bitmap[bitmap_num as usize] |= 1 << (31 - delta_n % 32);
            if delta_n + 1 > num_bits {
                num_bits = delta_n + 1;
            }
        }
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard refered to as "M"

        self.base.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        num_bits.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        for bitmap_element in &bitmap[..number_of_bitmap_elements] {
            bitmap_element.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        }
        Ok(())
    }
}

impl<'de, T> MappingReadByteOrdered<'de> for FragmentNumberSetSubmessageElement<T>
where
    T: FromIterator<FragmentNumber>,
{
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let base: FragmentNumber = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let num_bits: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let number_of_bitmap_elements = ((num_bits + 31) / 32) as usize; //In standard refered to as "M"
        let mut bitmap = [0; 8];
        for bitmap_i in bitmap.iter_mut().take(number_of_bitmap_elements) {
            *bitmap_i = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        }

        let mut set = Vec::with_capacity(256);
        for delta_n in 0..num_bits as usize {
            if (bitmap[delta_n / 32] & (1 << (31 - delta_n % 32))) == (1 << (31 - delta_n % 32)) {
                set.push(FragmentNumber(base.0 + delta_n as u32));
            }
        }
        Ok(Self {
            base,
            set: T::from_iter(set.into_iter()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping_traits::{from_bytes_le, to_bytes_le};

    #[test]
    fn serialize_fragment_number_max_gap() {
        let fragment_number_set = FragmentNumberSetSubmessageElement {
            base: FragmentNumber(2),
            set: vec![FragmentNumber(2), FragmentNumber(257)],
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&fragment_number_set).unwrap(), vec![
            2, 0, 0, 0, // bitmapBase: (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b_000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)
        ]);
    }

    #[test]
    fn deserialize_fragment_number_set_max_gap() {
        let expected = FragmentNumberSetSubmessageElement {
            base: FragmentNumber(2),
            set: vec![FragmentNumber(2), FragmentNumber(257)],
        };
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            2, 0, 0, 0, // bitmapBase: (unsigned long)
            0, 1, 0, 0, // numBits (unsigned long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_1000_0000, // bitmap[0] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[1] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[2] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[3] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[4] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[5] (long)
            0b_000_0000, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[6] (long)
            0b_000_0001, 0b_0000_0000, 0b_0000_0000, 0b_0000_0000, // bitmap[7] (long)

        ]).unwrap();
        assert_eq!(expected, result);
    }
}
