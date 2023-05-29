use crate::implementation::{
    rtps::messages::types::Time, rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

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
