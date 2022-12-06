use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::overall_structure::RtpsSubmessageHeader;

use crate::implementation::{
    rtps::messages::submessages::NackFragSubmessage, rtps_udp_psm::mapping_traits::MappingRead,
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for NackFragSubmessage {
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

impl<'de> MappingRead<'de> for NackFragSubmessage {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
