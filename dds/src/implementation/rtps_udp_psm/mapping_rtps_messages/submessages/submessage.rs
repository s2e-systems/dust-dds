use std::io::{Error, Write};

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::implementation::{
    rtps::messages::overall_structure::RtpsSubmessageHeader,
    rtps_udp_psm::mapping_traits::{MappingReadByteOrderInfoInData, MappingWriteByteOrderInfoInData},
};

pub trait MappingWriteSubmessage {
    fn submessage_header(&self) -> RtpsSubmessageHeader;
    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        writer: W,
    ) -> Result<(), Error>;
}

impl<T> MappingWriteByteOrderInfoInData for T
where
    T: MappingWriteSubmessage,
{
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut writer: W) -> Result<(), Error> {
        self.submessage_header().mapping_write_byte_order_info_in_data(&mut writer)?;
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
    ) -> Result<Self, Error>;
}

impl<'a, 'de: 'a, T> MappingReadByteOrderInfoInData<'de> for T
where
    T: MappingReadSubmessage<'de>,
{
    fn mapping_read_byte_order_info_in_data(buf: &mut &'de [u8]) -> Result<Self, Error> {
        let header: RtpsSubmessageHeader = MappingReadByteOrderInfoInData::mapping_read_byte_order_info_in_data(buf)?;
        if header.flags[0] {
            Self::mapping_read_submessage::<LittleEndian>(buf, header)
        } else {
            Self::mapping_read_submessage::<BigEndian>(buf, header)
        }
    }
}
