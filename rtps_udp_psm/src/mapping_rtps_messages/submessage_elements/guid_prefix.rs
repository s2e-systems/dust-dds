use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::{
    mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
    messages::submessage_elements::GuidPrefixSubmessageElementPsm,
};

impl MappingWriteByteOrdered for GuidPrefixSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GuidPrefixSubmessageElementPsm {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}
#[cfg(test)]
mod tests {
    use rust_rtps_pim::structure::types::GuidPrefix;

    use super::*;
    use crate::mapping_traits::{from_bytes_le, to_bytes_le};

    #[test]
    fn serialize_guid_prefix() {
        let data = GuidPrefixSubmessageElementPsm {
            value: GuidPrefix([1; 12]),
        };
        #[rustfmt::skip]
        assert_eq!(to_bytes_le(&data).unwrap(), vec![
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]);
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = GuidPrefixSubmessageElementPsm {
            value: GuidPrefix([1; 12]),
        };
        #[rustfmt::skip]
        assert_eq!(expected, from_bytes_le(&[
            1, 1, 1, 1,
            1, 1, 1, 1,
            1, 1, 1, 1,
        ]).unwrap());
    }
}
