use std::{
    convert::TryInto,
    io::{Error, Write},
};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::{messages::submessage_elements::EntityIdSubmessageElement, types::EntityId},
    rtps_udp_psm::mapping_traits::{
        MappingReadByteOrdered, MappingWriteByteOrdered, NumberOfBytes,
    },
};

impl MappingWriteByteOrdered for EntityId {
    fn mapping_write_byte_ordered<W: std::io::Write, B: byteorder::ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), std::io::Error> {
        Into::<[u8; 3]>::into(self.entity_key()).mapping_write_byte_ordered::<_, B>(&mut writer)?;
        Into::<u8>::into(self.entity_kind()).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for EntityId {
    fn mapping_read_byte_ordered<B>(buf: &mut &'de [u8]) -> Result<Self, Error>
    where
        B: ByteOrder,
    {
        let entity_key: [u8; 3] = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let entity_kind: u8 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(EntityId::new(entity_key, entity_kind.try_into().unwrap()))
    }
}

impl MappingWriteByteOrdered for EntityIdSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for EntityIdSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}
impl NumberOfBytes for EntityIdSubmessageElement {
    fn number_of_bytes(&self) -> usize {
        4
    }
}
