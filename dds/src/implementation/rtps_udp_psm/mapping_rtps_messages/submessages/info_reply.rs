use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::submessages::InfoReplySubmessage,
    rtps_udp_psm::mapping_traits::{MappingRead, MappingWriteByteOrderInfoInData},
};

impl MappingWriteByteOrderInfoInData for InfoReplySubmessage {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'de> MappingRead<'de> for InfoReplySubmessage {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
