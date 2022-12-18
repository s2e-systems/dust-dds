use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::types::Time,
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for Time {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        self.seconds()
            .mapping_write_byte_ordered::<_, B>(&mut writer)?;
        self.fraction()
            .mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for Time {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let seconds: i32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        let fraction: u32 = MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?;
        Ok(Time::new(seconds, fraction))
    }
}
