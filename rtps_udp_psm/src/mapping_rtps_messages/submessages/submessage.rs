use std::io::Write;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use rust_rtps_pim::messages::overall_structure::RtpsSubmessageHeader;

use crate::{deserialize::MappingRead, serialize::MappingWrite};


pub trait MappingWriteSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader;
    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        writer: W,
    ) -> crate::serialize::Result;
}

impl<T> MappingWrite for T
where
    T: MappingWriteSubmessage,
{
    fn mapping_write<W: Write>(&self, mut writer: W) -> crate::serialize::Result {
        self.submessage_header().mapping_write(&mut writer)?;
        if self.submessage_header().flags[0] {
            self.mapping_write_submessage_elements::<_, LittleEndian>(&mut writer)
        } else {
            self.mapping_write_submessage_elements::<_, BigEndian>(&mut writer)
        }
    }
}

pub trait MappingReadSubmessage<'de>: Sized {
    fn mapping_read_submessage<B: ByteOrder>(
        buf: &mut &'de [u8],
        header: RtpsSubmessageHeader,
    ) -> crate::deserialize::Result<Self>;
}

impl<'a, 'de: 'a, T> MappingRead<'de> for T
where
    T: MappingReadSubmessage<'de>,
{
    fn mapping_read(buf: &mut &'de [u8]) -> crate::deserialize::Result<Self> {
        let header: RtpsSubmessageHeader = MappingRead::mapping_read(buf)?;
        if header.flags[0] {
            Self::mapping_read_submessage::<LittleEndian>(buf, header)
        } else {
            Self::mapping_read_submessage::<BigEndian>(buf, header)
        }
    }
}
