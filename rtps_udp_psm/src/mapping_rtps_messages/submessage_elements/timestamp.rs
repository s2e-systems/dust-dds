use std::io::{Error, Write};

use byteorder::ByteOrder;
use rust_rtps_psm::messages::submessage_elements::TimestampSubmessageElementPsm;

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

impl MappingWriteByteOrdered for TimestampSubmessageElementPsm {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.value.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for TimestampSubmessageElementPsm {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(Self {
            value: MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        })
    }
}
