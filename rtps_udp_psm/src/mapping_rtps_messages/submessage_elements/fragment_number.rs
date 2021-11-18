use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{submessage_elements::FragmentNumberSubmessageElement, types::FragmentNumber}};

use crate::{
    deserialize::{self, MappingReadByteOrdered},
    serialize::{self, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for FragmentNumber {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for FragmentNumber {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(MappingReadByteOrdered::read_byte_ordered::<B>(buf)?))
    }
}

impl MappingWriteByteOrdered for FragmentNumberSubmessageElement {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for FragmentNumberSubmessageElement {
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
    fn serialize_fragment_number() {
        let data = FragmentNumberSubmessageElement { value: FragmentNumber(7) };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                7, 0, 0, 0, // (unsigned long)
            ]
        );
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumberSubmessageElement { value: FragmentNumber(7) };
        assert_eq!(
            expected,
            from_bytes_le(&[
                7, 0, 0, 0, // (unsigned long)
            ])
            .unwrap()
        );
    }
}
