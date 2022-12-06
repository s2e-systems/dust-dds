use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::{
    rtps::messages::submessages::HeartbeatFragSubmessage, rtps_udp_psm::mapping_traits::MappingReadByteOrderInfoInData,
};

use super::submessage::MappingWriteSubmessage;

impl MappingWriteSubmessage for HeartbeatFragSubmessage {
    fn submessage_header(
        &self,
    ) -> crate::implementation::rtps::messages::overall_structure::RtpsSubmessageHeader {
        todo!()
    }

    fn mapping_write_submessage_elements<W: Write, B: ByteOrder>(
        &self,
        _writer: W,
    ) -> Result<(), Error> {
        todo!()
    }
}

impl<'de> MappingReadByteOrderInfoInData<'de> for HeartbeatFragSubmessage {
    fn mapping_read_byte_order_info_in_data(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
