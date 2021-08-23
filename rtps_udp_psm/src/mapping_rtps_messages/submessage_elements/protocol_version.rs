use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    messages::submessage_elements::ProtocolVersionSubmessageElement,
    structure::types::ProtocolVersion,
};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for ProtocolVersion {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.major.serialize::<_, B>(&mut writer)?;
        self.minor.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for ProtocolVersion {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            major: Deserialize::deserialize::<B>(buf)?,
            minor: Deserialize::deserialize::<B>(buf)?,
        })
    }
}

impl Serialize for ProtocolVersionSubmessageElement {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for ProtocolVersionSubmessageElement {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self { value: Deserialize::deserialize::<B>(buf)? })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_protocol_version() {
        let data = ProtocolVersionSubmessageElement {
            value: ProtocolVersion { major: 2, minor: 3 },
        };
        assert_eq!(to_bytes_le(&data).unwrap(), vec![2, 3]);
    }

    #[test]
    fn deserialize_protocol_version() {
        let expected = ProtocolVersionSubmessageElement {
            value: ProtocolVersion { major: 2, minor: 3 },
        };
        assert_eq!(expected, from_bytes_le(&[2, 3]).unwrap());
    }
}
