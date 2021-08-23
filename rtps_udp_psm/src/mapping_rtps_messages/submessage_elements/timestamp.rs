use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{submessage_elements::TimestampSubmessageElement, types::Time}};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for TimestampSubmessageElement {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        let seconds = (self.value.0 >> 32) as i32;
        let fraction = self.value.0 as i32;
        seconds.serialize::<_, B>(&mut writer)?;
        fraction.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for TimestampSubmessageElement {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let seconds: i32 = Deserialize::deserialize::<B>(buf)?;
        let fraction: u32 = Deserialize::deserialize::<B>(buf)?;
        let value = Time(((seconds as u64) << 32) + fraction as u64);
        Ok(Self{value})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_time_stamp() {
        let data = TimestampSubmessageElement { value: Time(7) };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![
            0, 0, 0, 0, // seconds (long)
            7, 0, 0, 0, // fraction (unsinged long)
        ]);
    }

    #[test]
    fn deserialize_time_stamp() {
        let expected = TimestampSubmessageElement { value: Time(7) };
        assert_eq!(expected, from_bytes_le(&[
            0, 0, 0, 0, // seconds (long)
            7, 0, 0, 0, // fraction (unsinged long)
        ]).unwrap());
    }
}
