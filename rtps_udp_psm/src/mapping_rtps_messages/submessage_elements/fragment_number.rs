use std::io::{Error, Write};

use byteorder::ByteOrder;
use dds_transport::messages::submessage_elements::FragmentNumberSubmessageElement;

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

impl MappingWriteByteOrdered for FragmentNumberSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for FragmentNumberSubmessageElement {
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
    fn serialize_fragment_number() {
        let data = FragmentNumberSubmessageElement { value: 7 };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                7, 0, 0, 0, // (unsigned long)
            ]
        );
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumberSubmessageElement { value: 7 };
        assert_eq!(
            expected,
            from_bytes_le(&[
                7, 0, 0, 0, // (unsigned long)
            ])
            .unwrap()
        );
    }
}
