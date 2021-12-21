use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_pim::messages::submessage_elements::EntityIdSubmessageElement;
use rust_rtps_psm::messages::submessage_elements::EntityIdSubmessageElementPsm;

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

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

impl MappingWriteByteOrdered for EntityIdSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
impl<'de> MappingReadByteOrdered<'de> for EntityIdSubmessageElementPsm {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}