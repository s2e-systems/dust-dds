use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::{overall_structure::RtpsSubmessageHeader, submessages::DataFragSubmessage},
    rtps_udp_psm::mapping_traits::MappingRead,
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for DataFragSubmessage<'_> {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        _writer: W,
    ) -> Result<(), Error> {
        todo!()
    }
}
impl<'a, 'de: 'a> MappingRead<'de> for DataFragSubmessage<'a> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
