use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::{
    behavior::types::Duration,
    messages::types::{GroupDigest, Time},
    structure::types::Locator,
};

use crate::{
    deserialize::MappingReadByteOrdered,
    serialize::{MappingWriteByteOrdered, NumberOfBytes},
};

impl MappingWriteByteOrdered for Time {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let seconds = (self.0 >> 32) as i32;
        let fraction = self.0 as i32;
        seconds.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        fraction.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Time {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let seconds: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let fraction: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Self(((seconds as u64) << 32) + fraction as u64))
    }
}

impl MappingWriteByteOrdered for Duration {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.seconds
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.fraction
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Duration {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            seconds: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            fraction: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

impl NumberOfBytes for Duration {
    fn number_of_bytes(&self) -> usize {
        8
    }
}

impl MappingWriteByteOrdered for Locator {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.kind()
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.port()
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.address()
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Locator {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let kind = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let port = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let address = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Self::new(kind, port, address))
    }
}

impl NumberOfBytes for Locator {
    fn number_of_bytes(&self) -> usize {
        24
    }
}

impl MappingWriteByteOrdered for GroupDigest {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.0.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GroupDigest {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_time() {
        let time = Time(7);
        assert_eq!(
            to_bytes_le(&time).unwrap(),
            vec![
                0, 0, 0, 0, // seconds (long)
                7, 0, 0, 0, // fraction (unsinged long)
            ]
        );
    }

    #[test]
    fn deserialize_time() {
        let expected = Time(7);
        assert_eq!(
            expected,
            from_bytes_le(&[
                0, 0, 0, 0, // seconds (long)
                7, 0, 0, 0, // fraction (unsinged long)
            ])
            .unwrap()
        );
    }

    #[test]
    fn serialize_duration() {
        let time = Duration {
            seconds: 7,
            fraction: 45,
        };
        assert_eq!(
            to_bytes_le(&time).unwrap(),
            vec![
                7, 0, 0, 0, // seconds (long)
                45, 0, 0, 0, // fraction (unsinged long)
            ]
        );
    }

    #[test]
    fn deserialize_duration() {
        let expected = Duration {
            seconds: 7,
            fraction: 45,
        };
        assert_eq!(
            expected,
            from_bytes_le(&[
                7, 0, 0, 0, // seconds (long)
                45, 0, 0, 0, // fraction (unsinged long)
            ])
            .unwrap()
        );
    }

    #[test]
    fn serialize_locator() {
        let locator = Locator::new(1, 2, [3; 16]);
        assert_eq!(
            to_bytes_le(&locator).unwrap(),
            vec![
                1, 0, 0, 0, // kind (long)
                2, 0, 0, 0, // port (unsigned long)
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
                3, 3, 3, 3, // address (octet[16])
            ]
        );
    }

    #[test]
    fn deserialize_locator() {
        let expected = Locator::new(1, 2, [3; 16]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            1, 0, 0, 0, // kind (long)
            2, 0, 0, 0, // port (unsigned long)
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])
            3, 3, 3, 3, // address (octet[16])

        ]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_group_digest() {
        let locator = GroupDigest([3; 4]);
        assert_eq!(to_bytes_le(&locator).unwrap(), vec![3, 3, 3, 3,]);
    }

    #[test]
    fn deserialize_group_digest() {
        let expected = GroupDigest([3; 4]);
        #[rustfmt::skip]
        let result = from_bytes_le(&[
            3, 3, 3, 3,

        ]).unwrap();
        assert_eq!(expected, result);
    }
}
