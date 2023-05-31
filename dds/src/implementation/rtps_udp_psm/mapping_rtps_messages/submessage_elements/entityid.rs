use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::types::EntityId,
    rtps_udp_psm::mapping_traits::{MappingWriteByteOrdered, NumberOfBytes},
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

impl NumberOfBytes for EntityId {
    fn number_of_bytes(&self) -> usize {
        4
    }
}
