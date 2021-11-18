use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::{submessage_elements::CountSubmessageElement, types::Count};

use crate::{deserialize::{self, MappingReadByteOrdered}, serialize::{self, NumberOfBytes, MappingWriteByteOrdered}};

impl MappingWriteByteOrdered for Count {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for Count {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(MappingReadByteOrdered::read_byte_ordered::<B>(buf)?))
    }
}
impl NumberOfBytes for Count {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

impl MappingWriteByteOrdered for CountSubmessageElement {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for CountSubmessageElement {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            value: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

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
