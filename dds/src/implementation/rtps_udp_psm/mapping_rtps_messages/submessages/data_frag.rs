use std::io::{Error, Write};

use crate::implementation::{
    rtps::messages::submessages::DataFragSubmessage,
    rtps_udp_psm::mapping_traits::{MappingRead, MappingWriteByteOrderInfoInData},
};

impl MappingWriteByteOrderInfoInData for DataFragSubmessage<'_> {
    fn mapping_write_byte_order_info_in_data<W: Write>(&self, mut _writer: W) -> Result<(), Error> {
        todo!()
    }
}
impl<'a, 'de: 'a> MappingRead<'de> for DataFragSubmessage<'a> {
    fn mapping_read(_buf: &mut &'de [u8]) -> Result<Self, Error> {
        todo!()
    }
}
