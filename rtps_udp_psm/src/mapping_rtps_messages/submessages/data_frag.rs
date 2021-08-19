use rust_rtps_pim::messages::submessages::DataFragSubmessage;

use crate::{deserialize::Deserialize, serialize::Serialize};

use byteorder::ByteOrder;
use std::io::Write;


impl<'a, S> Serialize for DataFragSubmessage<'a, S> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de:'a, 'a, S> Deserialize<'de> for DataFragSubmessage<'a, S> {
    fn deserialize<B: ByteOrder>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> {
        todo!()
    }
}