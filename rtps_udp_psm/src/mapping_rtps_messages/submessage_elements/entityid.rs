use std::io::Write;

use byteorder::ByteOrder;
use rust_rtps_pim::{
    messages::submessage_elements::EntityIdSubmessageElement,
    structure::types::{EntityId, EntityKind},
};

use crate::{
    deserialize::{self, Deserialize},
    serialize::{self, Serialize},
};

// Table 9.1 - entityKind octet of an EntityId_t
pub const USER_DEFINED_UNKNOWN: u8 = 0x00;
pub const BUILT_IN_UNKNOWN: u8 = 0xc0;
pub const BUILT_IN_PARTICIPANT: u8 = 0xc1;
pub const USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
pub const BUILT_IN_WRITER_WITH_KEY: u8 = 0xc2;
pub const USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
pub const BUILT_IN_WRITER_NO_KEY: u8 = 0xc3;
pub const USER_DEFINED_READER_WITH_KEY: u8 = 0x07;
pub const BUILT_IN_READER_WITH_KEY: u8 = 0xc7;
pub const USER_DEFINED_READER_NO_KEY: u8 = 0x04;
pub const BUILT_IN_READER_NO_KEY: u8 = 0xc4;
pub const USER_DEFINED_WRITER_GROUP: u8 = 0x08;
pub const BUILT_IN_WRITER_GROUP: u8 = 0xc8;
pub const USER_DEFINED_READER_GROUP: u8 = 0x09;
pub const BUILT_IN_READER_GROUP: u8 = 0xc9;

impl Serialize for EntityIdSubmessageElement {
    fn serialize<W: Write, B: ByteOrder>(&self, mut writer: W) -> serialize::Result {
        self.value.entity_key().serialize::<_, B>(&mut writer)?;
        let entity_kind = match self.value.entity_kind() {
            EntityKind::UserDefinedUnknown => USER_DEFINED_UNKNOWN,
            EntityKind::BuiltInUnknown => BUILT_IN_UNKNOWN,
            EntityKind::BuiltInParticipant => BUILT_IN_PARTICIPANT,
            EntityKind::UserDefinedWriterWithKey => USER_DEFINED_WRITER_WITH_KEY,
            EntityKind::BuiltInWriterWithKey => BUILT_IN_WRITER_WITH_KEY,
            EntityKind::UserDefinedWriterNoKey => USER_DEFINED_WRITER_NO_KEY,
            EntityKind::BuiltInWriterNoKey => BUILT_IN_WRITER_NO_KEY,
            EntityKind::UserDefinedReaderWithKey => USER_DEFINED_READER_WITH_KEY,
            EntityKind::BuiltInReaderWithKey => BUILT_IN_READER_WITH_KEY,
            EntityKind::UserDefinedReaderNoKey => USER_DEFINED_READER_NO_KEY,
            EntityKind::BuiltInReaderNoKey => BUILT_IN_READER_NO_KEY,
            EntityKind::UserDefinedWriterGroup => USER_DEFINED_WRITER_GROUP,
            EntityKind::BuiltInWriterGroup => BUILT_IN_WRITER_GROUP,
            EntityKind::UserDefinedReaderGroup => USER_DEFINED_READER_GROUP,
            EntityKind::BuiltInReaderGroup => BUILT_IN_READER_GROUP,
        };
        entity_kind.serialize::<_, B>(&mut writer)
    }
}
impl<'de> Deserialize<'de> for EntityIdSubmessageElement {
    fn deserialize<B: ByteOrder>(buf: &mut &'de [u8]) -> deserialize::Result<Self> {
        let entity_key = Deserialize::deserialize::<B>(buf)?;
        let entity_kind: u8 = Deserialize::deserialize::<B>(buf)?;
        let entity_kind = match entity_kind {
            USER_DEFINED_UNKNOWN => EntityKind::UserDefinedUnknown,
            BUILT_IN_UNKNOWN => EntityKind::BuiltInUnknown,
            BUILT_IN_PARTICIPANT => EntityKind::BuiltInParticipant,
            USER_DEFINED_WRITER_WITH_KEY => EntityKind::UserDefinedWriterWithKey,
            BUILT_IN_WRITER_WITH_KEY => EntityKind::BuiltInWriterWithKey,
            USER_DEFINED_WRITER_NO_KEY => EntityKind::UserDefinedWriterNoKey,
            BUILT_IN_WRITER_NO_KEY => EntityKind::BuiltInWriterNoKey,
            USER_DEFINED_READER_WITH_KEY => EntityKind::UserDefinedReaderWithKey,
            BUILT_IN_READER_WITH_KEY => EntityKind::BuiltInReaderWithKey,
            USER_DEFINED_READER_NO_KEY => EntityKind::UserDefinedReaderNoKey,
            BUILT_IN_READER_NO_KEY => EntityKind::BuiltInReaderNoKey,
            USER_DEFINED_WRITER_GROUP => EntityKind::UserDefinedWriterGroup,
            BUILT_IN_WRITER_GROUP => EntityKind::BuiltInWriterGroup,
            USER_DEFINED_READER_GROUP => EntityKind::UserDefinedReaderGroup,
            BUILT_IN_READER_GROUP => EntityKind::BuiltInReaderGroup,
            _ => {
                return deserialize::Result::Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "EntityKind unknown",
                ))
            }
        };
        let value = EntityId::new(entity_key, entity_kind);
        Ok(Self { value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deserialize::from_bytes_le;
    use crate::serialize::to_bytes_le;

    #[test]
    fn serialize_entity_id() {
        let entity_id = EntityIdSubmessageElement {
            value: EntityId::new(
                [1, 2, 3],
                rust_rtps_pim::structure::types::EntityKind::BuiltInParticipant,
            ),
        };
        assert_eq!(to_bytes_le(&entity_id).unwrap(), [1, 2, 3, 0xc1]);
    }

    #[test]
    fn deserialize_locator() {
        let expected = EntityIdSubmessageElement {
            value: EntityId::new(
                [1, 2, 3],
                rust_rtps_pim::structure::types::EntityKind::BuiltInParticipant,
            ),
        };
        let result: EntityIdSubmessageElement = from_bytes_le(&[1, 2, 3, 0xc1]).unwrap();
        assert_eq!(expected, result);
    }
}
