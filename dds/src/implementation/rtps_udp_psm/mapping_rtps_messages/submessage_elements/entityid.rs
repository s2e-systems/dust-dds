use std::{
    convert::TryInto,
    io::{Error, Write},
};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::{messages::submessage_elements::EntityIdSubmessageElement, types::{EntityId, EntityKind}},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for EntityIdSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        Into::<[u8; 3]>::into(self.value.entity_key())
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        Into::<u8>::into(self.value.entity_kind()).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for EntityIdSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let entity_key: [u8; 3] = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let entity_kind: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let entity_kind: EntityKind = entity_kind
            .try_into()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "EntityKind not valid"))?;
        Ok(Self {
            value: EntityId::new(entity_key, entity_kind),
        })
    }
}
impl NumberOfBytes for EntityIdSubmessageElement {
    fn number_of_bytes(&self) -> usize {
        4
    }
}
