use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::messages::types::FragmentNumber;
use rust_rtps_psm::messages::submessage_elements::FragmentNumberSubmessageElementPsm;

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

impl MappingWriteByteOrdered for FragmentNumber {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.0.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for FragmentNumber {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}

impl MappingWriteByteOrdered for FragmentNumberSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for FragmentNumberSubmessageElementPsm {
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
        let data = FragmentNumberSubmessageElementPsm {
            value: FragmentNumber(7),
        };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                7, 0, 0, 0, // (unsigned long)
            ]
        );
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumberSubmessageElementPsm {
            value: FragmentNumber(7),
        };
        assert_eq!(
            expected,
            from_bytes_le(&[
                7, 0, 0, 0, // (unsigned long)
            ])
            .unwrap()
        );
    }
}
