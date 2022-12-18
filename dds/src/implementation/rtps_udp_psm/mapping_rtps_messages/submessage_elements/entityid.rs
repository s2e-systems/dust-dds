use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::{EntityId, EntityKey, EntityKind},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for EntityId {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        Into::<[u8; 3]>::into(self.entity_key()).mapping_write_byte_ordered::<_, B>(&mut writer)?;
        Into::<u8>::into(self.entity_kind()).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for EntityId {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let entity_key: [u8; 3] = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let entity_kind: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(EntityId::new(
            EntityKey::new(entity_key),
            EntityKind::new(entity_kind),
        ))
    }
}
impl NumberOfBytes for EntityId {
    fn number_of_bytes(&self) -> usize {
        4
    }
}
