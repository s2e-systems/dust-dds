use rust_rtps_pim::messages::submessages::AckNackSubmessage;

use crate::serialize::Serialize;

use byteorder::ByteOrder;
use std::io::Write;


impl<S> Serialize for AckNackSubmessage<S> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de, S> crate::deserialize::Deserialize<'de> for AckNackSubmessage<S> {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}