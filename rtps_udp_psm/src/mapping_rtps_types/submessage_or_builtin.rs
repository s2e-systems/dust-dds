use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{types::Time}};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_time() {
        let time = Time(7);
        assert_eq!(to_bytes_le(&time).unwrap(), vec![
            0, 0, 0, 0, // seconds (long)
            7, 0, 0, 0, // fraction (unsinged long)
        ]);
    }

    #[test]
    fn deserialize_time_stamp() {
        let expected = Time(7);
        assert_eq!(expected, from_bytes_le(&[
            0, 0, 0, 0, // seconds (long)
            7, 0, 0, 0, // fraction (unsinged long)
        ]).unwrap());
    }
}
