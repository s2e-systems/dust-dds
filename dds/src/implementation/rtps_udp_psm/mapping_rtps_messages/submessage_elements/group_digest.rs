use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::types::{GroupDigest, ULong, UShort},
    rtps_udp_psm::mapping_traits::{MappingReadByteOrdered, MappingWriteByteOrdered},
};

impl MappingWriteByteOrdered for GroupDigest {
    fn mapping_write_byte_ordered<W: Write, B: ByteOrder>(
        &self,
        mut writer: W,
    ) -> Result<(), Error> {
        <[u8; 4]>::from(*self).mapping_write_byte_ordered::<_, B>(&mut writer)
    }
}

impl<'de> MappingReadByteOrdered<'de> for GroupDigest {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(GroupDigest::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
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

impl<'de> MappingReadByteOrdered<'de> for ULong {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(ULong::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
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

impl<'de> MappingReadByteOrdered<'de> for UShort {
    fn mapping_read_byte_ordered<B: ByteOrder>(buf: &mut &'de [u8]) -> Result<Self, Error> {
        Ok(UShort::new(
            MappingReadByteOrdered::mapping_read_byte_ordered::<B>(buf)?,
        ))
    }
}


