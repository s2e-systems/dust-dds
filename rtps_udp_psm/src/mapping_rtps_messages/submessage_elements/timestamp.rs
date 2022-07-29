use std::io::{Error, Write};

use byteorder::ByteOrder;
use rtps_pim::messages::submessage_elements::TimestampSubmessageElement;

use crate::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered};

const SEC_IN_NANOSEC: u64 = 1000000000;

impl MappingWriteByteOrdered for TimestampSubmessageElement {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        let seconds = (self.value / SEC_IN_NANOSEC) as i32;
        let fraction = (self.value >> 32) as u32;
        seconds.mapping_write_byte_ordered::<_, B>(&mut writer)?;
        fraction.mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for TimestampSubmessageElement {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let seconds: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let fraction: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let value =
            ((seconds as u64) * SEC_IN_NANOSEC) + ((fraction as u64) << 32) / SEC_IN_NANOSEC;

        Ok(Self { value })
    }
}
