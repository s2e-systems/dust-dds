use std::io::{Error, Write};

use byteorder::ByteOrder;

use crate::implementation::rtps::messages::{
    overall_structure::RtpsSubmessageHeader, submessages::InfoReplySubmessage,
};

use super::submessage::{MappingReadSubmessage, MappingWriteSubmessage};

impl MappingWriteSubmessage for InfoReplySubmessage {
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

impl<'de> MappingReadSubmessage<'de> for InfoReplySubmessage {
    fn mapping_read_submessage<B: ByteOrder>(
        _buf: &mut &'de [u8],
        _header: RtpsSubmessageHeader,
    ) -> Result<Self, Error> {
        todo!()
    }
}
