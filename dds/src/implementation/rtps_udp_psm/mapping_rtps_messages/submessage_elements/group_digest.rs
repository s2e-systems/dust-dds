use crate::implementation::{
    rtps::messages::types::{GroupDigest, ULong, UShort},
    rtps_udp_psm::mapping_traits::MappingWriteByteOrdered,
};
use byteorder::ByteOrder;
use std::io::{Error, Write};

impl MappingWriteByteOrdered for GroupDigest {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <[u8; 4]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl MappingWriteByteOrdered for ULong {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <u32>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl MappingWriteByteOrdered for UShort {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <u16>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}
