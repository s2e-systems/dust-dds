use std::io::{Error, Write};

use byteorder::ByteOrder;
use rtps_pim::{
    messages::submessage_elements::ProtocolVersionSubmessageElement,
    structure::types::ProtocolVersion,
};

use crate::mapping_traits::{
    MappingRead, MappingReadByteOrdered, MappingWrite, MappingWriteByteOrdered, NumberOfBytes,
};

impl MappingWriteByteOrdered for ProtocolVersion {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.major.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.minor.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl MappingWrite for ProtocolVersion {
    fn mapping_write<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.major.mapping_write(&mut writer)?;
        self.minor.mapping_write(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ProtocolVersion {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            major: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
            minor: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}

impl<'de> MappingRead<'de> for ProtocolVersion {
    fn mapping_read(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            major: MappingRead::mapping_read(buf)?,
            minor: MappingRead::mapping_read(buf)?,
        })
    }
}

impl NumberOfBytes for ProtocolVersion {
    fn number_of_bytes(&self) -> usize {
        2
    }
}

impl MappingWriteByteOrdered for ProtocolVersionSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for ProtocolVersionSubmessageElement {
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
