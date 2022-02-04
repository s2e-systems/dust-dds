use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::{
    mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
    messages::submessage_elements::GroupDigestSubmessageElementPsm,
};

impl MappingWriteByteOrdered for GroupDigestSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GroupDigestSubmessageElementPsm {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}
