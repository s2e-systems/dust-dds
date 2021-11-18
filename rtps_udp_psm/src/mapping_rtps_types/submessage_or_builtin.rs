use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    behavior::types::Duration,
    messages::types::{GroupDigest, Time},
    structure::types::Locator,
};

use crate::{deserialize::{self, MappingReadByteOrdered}, serialize::{self, NumberOfBytes, MappingWriteByteOrdered}};

impl MappingWriteByteOrdered for Time {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        let seconds = (self.0 >> 32) as i32;
        let fraction = self.0 as i32;
        seconds.write_byte_ordered::<_, B>(&mut writer)?;
        fraction.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Time {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let seconds: i32 = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        let fraction: u32 = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        Ok(Self(((seconds as u64) << 32) + fraction as u64))
    }
}

impl MappingWriteByteOrdered for Duration {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.seconds.write_byte_ordered::<_, B>(&mut writer)?;
        self.fraction.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Duration {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            seconds: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
            fraction: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
        })
    }
}

impl NumberOfBytes for Duration {
    fn number_of_bytes(&self) -> usize {
        8
    }
}

impl MappingWriteByteOrdered for Locator {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.kind().write_byte_ordered::<_, B>(&mut writer)?;
        self.port().write_byte_ordered::<_, B>(&mut writer)?;
        self.address().write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Locator {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let kind = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        let port = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        let address = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        Ok(Self::new(kind, port, address))
    }
}

impl NumberOfBytes for Locator {
    fn number_of_bytes(&self) -> usize {
        24
    }
}

impl MappingWriteByteOrdered for GroupDigest {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GroupDigest {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(MappingReadByteOrdered::read_byte_ordered::<B>(buf)?))
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
