use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::messages::{submessage_elements::GroupDigestSubmessageElement, types::{Count, GroupDigest}};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for GroupDigest {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> serialize::Result {
        Ok(())
    }
}
impl<'de> Deserialize<'de> for GroupDigest {
    fn deserialize<B: ByteOrder>(_buf: &mut &'de[u8]) -> deserialize::Result<Self> {
        Ok(())
    }
}

impl Serialize for GroupDigestSubmessageElement {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for GroupDigestSubmessageElement {
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
    fn serialize_guid_prefix() {
        let data = GroupDigestSubmessageElement{ value: ()};
        assert_eq!(to_bytes_le(&data).unwrap(), vec![]);
    }

    #[test]
    fn deserialize_guid_prefix() {
        let expected = GroupDigestSubmessageElement{ value: ()};
        assert_eq!(expected, from_bytes_le(&[
        ]).unwrap());
    }
}