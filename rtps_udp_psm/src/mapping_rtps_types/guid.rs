use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::structure::types::{EntityId, Guid};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

impl Serialize for EntityId {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.entity_key.serialize::<_, B>(&mut writer)?;
        self.entity_kind.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for EntityId {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let entity_key = Deserialize::deserialize::<B>(buf)?;
        let entity_kind = Deserialize::deserialize::<B>(buf)?;
        Ok(Self {
            entity_key,
            entity_kind,
        })
    }
}

impl Serialize for Guid {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.prefix.serialize::<_, B>(&mut writer)?;
        self.entity_id.serialize::<_, B>(&mut writer)
    }
}

impl<'de> Deserialize<'de> for Guid {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        Ok(Self {
            prefix: Deserialize::deserialize::<B>(buf)?,
            entity_id: Deserialize::deserialize::<B>(buf)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_rtps_pim::structure::types::{BUILT_IN_PARTICIPANT, EntityId, USER_DEFINED_READER_NO_KEY};

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
            prefix: [3; 12],
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
            prefix: [3; 12],
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
