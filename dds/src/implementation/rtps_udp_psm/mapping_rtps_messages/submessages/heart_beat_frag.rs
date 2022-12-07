use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::{submessages::HeartbeatFragSubmessage, overall_structure::RtpsSubmessageHeader};

use super::submessage::{MappingWriteSubmessage, MappingReadSubmessage};

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

impl<'de> MappingReadSubmessage<'de> for HeartbeatFragSubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        _buf: &mut &'de [u8],
        _header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        todo!()
    }
}
