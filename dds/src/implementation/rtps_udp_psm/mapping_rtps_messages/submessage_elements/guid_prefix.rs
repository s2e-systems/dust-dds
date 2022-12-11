use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::{messages::submessage_elements::GuidPrefixSubmessageElement, types::GuidPrefix},
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for GuidPrefix {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> Result<(), Error> {
        <[u8; 12]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GuidPrefix {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self::new(MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?))
    }
}

impl MappingWriteByteOrdered for GuidPrefixSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GuidPrefixSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::implementation::rtps_udp_psm::mapping_traits::{from_bytes_le, to_bytes_le};

    use super::*;

    #[test]
    fn serialize_guid_prefix() {
        let data = GuidPrefixSubmessageElement { value: GuidPrefix::new([1; 12]) };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&data).unwrap(), vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]);
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = GuidPrefixSubmessageElement { value: GuidPrefix::new([1; 12]) };
        #[rustfmt::skip]
        assert_eq!(expected, from_bytes_le(&[
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]).unwrap());
    }
}
