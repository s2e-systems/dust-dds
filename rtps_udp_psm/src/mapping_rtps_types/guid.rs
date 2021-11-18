use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::structure::types::{EntityId, Guid, GuidPrefix};

use crate::{
    deserialize::{self, MappingReadByteOrdered, MappingRead},
    serialize::{self, MappingWrite, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for EntityId {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.entity_key.write_byte_ordered::<_, B>(&mut writer)?;
        self.entity_kind.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for EntityId {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let entity_key = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        let entity_kind = MappingReadByteOrdered::read_byte_ordered::<B>(buf)?;
        Ok(Self {
            entity_key,
            entity_kind,
        })
    }
}

impl MappingWriteByteOrdered for GuidPrefix {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.0.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GuidPrefix {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(MappingReadByteOrdered::read_byte_ordered::<B>(buf)?))
    }
}

impl MappingWrite for GuidPrefix {
    fn write<W: Write>(&self, writer: W) -> serialize::Result {
        self.0.write(writer)
    }
}

impl<'de> MappingRead<'de> for GuidPrefix {
    fn read(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self(MappingRead::read(buf)?))
    }
}

impl MappingWriteByteOrdered for Guid {
    fn write_byte_ordered<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.prefix.write_byte_ordered::<_, B>(&mut writer)?;
        self.entity_id.write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Guid {
    fn read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            prefix: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
            entity_id: MappingReadByteOrdered::read_byte_ordered::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::structure::types::{
        EntityId, GuidPrefix, BUILT_IN_PARTICIPANT, USER_DEFINED_READER_NO_KEY,
    };

    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_entity_id() {
        let entity_id = EntityId::new([1, 2, 3], BUILT_IN_PARTICIPANT);
        assert_eq!(to_bytes_le(&entity_id).unwrap(), [1, 2, 3, 0xc1]);
    }

    #[test]
    fn deserialize_locator() {
        let expected = EntityId::new([1, 2, 3], BUILT_IN_PARTICIPANT);
        let result = from_bytes_le(&[1, 2, 3, 0xc1]).unwrap();
        assert_eq!(expected, result);
    }

    #[test]
    fn serialize_guid() {
        let guid = Guid {
            prefix: GuidPrefix([3; 12]),
            entity_id: EntityId {
                entity_key: [1, 2, 3],
                entity_kind: USER_DEFINED_READER_NO_KEY,
            },
        };

        assert_eq!(
            to_bytes_le(&guid).unwrap(),
            [
                3, 3, 3, 3, // GuidPrefix
                3, 3, 3, 3, // GuidPrefix
                3, 3, 3, 3, // GuidPrefix
                1, 2, 3, 0x04 // EntityId
            ]
        );
    }

    #[test]
    fn deserialize_guid() {
        let expected = Guid {
            prefix: GuidPrefix([3; 12]),
            entity_id: EntityId {
                entity_key: [1, 2, 3],
                entity_kind: USER_DEFINED_READER_NO_KEY,
            },
        };

        assert_eq!(
            expected,
            from_bytes_le(&[
                3, 3, 3, 3, // GuidPrefix
                3, 3, 3, 3, // GuidPrefix
                3, 3, 3, 3, // GuidPrefix
                1, 2, 3, 0x04 // EntityId
            ])
            .unwrap()
        );
    }
}
