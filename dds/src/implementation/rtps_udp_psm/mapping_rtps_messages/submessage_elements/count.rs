use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::{messages::submessage_elements::CountSubmessageElement, types::Count},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for Count {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.as_ref()
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Count {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
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

impl NumberOfBytes for CountSubmessageElement {
    fn number_of_bytes(&self) -> usize {
        4
    }
}

#[cfg(test)]
mod tests {
    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le};

    use super::*;

    #[test]
    fn serialize_guid_prefix() {
        let data = CountSubmessageElement {
            value: Count::new(7),
        };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
            7, 0, 0,0 , //value (long)
        ]
        );
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = CountSubmessageElement {
            value: Count::new(7),
        };
        assert_eq!(
            expected,
            from_bytes_le(&[
            7, 0, 0,0 , //value (long)
        ])
            .unwrap()
        );
    }
}
