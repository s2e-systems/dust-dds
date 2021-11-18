use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::messages::{submessage_elements::CountSubmessageElement, types::Count};

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes};

impl MappingWriteByteOrdered for Count {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.0.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for Count {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}
impl NumberOfBytes for Count {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl MappingWriteByteOrdered for CountSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for CountSubmessageElement {
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
    fn serialize_guid_prefix() {
        let data = CountSubmessageElement { value: Count(7) };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
            7, 0, 0,0 , //value (long)
        ]
        );
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = CountSubmessageElement { value: Count(7) };
        assert_eq!(
            expected,
            from_bytes_le(&[
            7, 0, 0,0 , //value (long)
        ])
            .unwrap()
        );
    }
}
