use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{messages::{submessage_elements::FragmentNumberSubmessageElement, types::FragmentNumber}};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for FragmentNumber {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for FragmentNumber {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(Deserialize::deserialize::<B>(buf)?))
    }
}

impl Serialize for FragmentNumberSubmessageElement {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for FragmentNumberSubmessageElement {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            value: Deserialize::deserialize::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_fragment_number() {
        let data = FragmentNumberSubmessageElement { value: FragmentNumber(7) };
        assert_eq!(
            to_bytes_le(&data).unwrap(),
            vec![
                7, 0, 0, 0, // (unsigned long)
            ]
        );
    }

    #[test]
    fn deserialize_fragment_number() {
        let expected = FragmentNumberSubmessageElement { value: FragmentNumber(7) };
        assert_eq!(
            expected,
            from_bytes_le(&[
                7, 0, 0, 0, // (unsigned long)
            ])
            .unwrap()
        );
    }
}
