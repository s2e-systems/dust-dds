use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    messages::submessage_elements::ProtocolVersionSubmessageElement,
    structure::types::ProtocolVersion,
};

use crate::{deserialize::{self, MappingReadByteOrdered, MappingRead}, serialize::{self, MappingWrite, NumberOfBytes, MappingWriteByteOrdered}};

impl MappingWriteByteOrdered for ProtocolVersion {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.major.write_byte_ordered::<_, B>(&mut writer)?;
        self.minor.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl MappingWrite for ProtocolVersion {
    fn write<W: Write>(&self, mut writer: W) -> serialize::Result {
        self.major.write(&mut writer)?;
        self.minor.write(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ProtocolVersion {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            major: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
            minor: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
        })
    }
}

impl<'de> MappingRead<'de> for ProtocolVersion {
    fn read(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            major: MappingRead::read(buf)?,
            minor: MappingRead::read(buf)?,
        })
    }
}

impl NumberOfBytes for ProtocolVersion {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl MappingWriteByteOrdered for ProtocolVersionSubmessageElement {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ProtocolVersionSubmessageElement {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self { value: MappingReadByteOrdered::read_byte_ordered::<B>(buf)? })
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
