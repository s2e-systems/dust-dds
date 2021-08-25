use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    behavior::types::Duration,
    messages::types::{GroupDigest, Time},
    structure::types::Locator,
};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for Time {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        let seconds = (self.0 >> 32) as i32;
        let fraction = self.0 as i32;
        seconds.serialize::<_, B>(&mut writer)?;
        fraction.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for Time {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let seconds: i32 = Deserialize::deserialize::<B>(buf)?;
        let fraction: u32 = Deserialize::deserialize::<B>(buf)?;
        Ok(Self(((seconds as u64) << 32) + fraction as u64))
    }
}

impl Serialize for Duration {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.seconds.serialize::<_, B>(&mut writer)?;
        self.fraction.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for Duration {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            seconds: Deserialize::deserialize::<B>(buf)?,
            fraction: Deserialize::deserialize::<B>(buf)?,
        })
    }
}

impl Serialize for Locator {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.kind().serialize::<_, B>(&mut writer)?;
        self.port().serialize::<_, B>(&mut writer)?;
        self.address().serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for Locator {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let kind = Deserialize::deserialize::<B>(buf)?;
        let port = Deserialize::deserialize::<B>(buf)?;
        let address = Deserialize::deserialize::<B>(buf)?;
        Ok(Self::new(kind, port, address))
    }
}

impl Serialize for GroupDigest {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for GroupDigest {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(Deserialize::deserialize::<B>(buf)?))
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
