use rust_rtps_pim::messages::submessages::HeartbeatSubmessage;

use crate::{deserialize::Deserialize, serialize::Serialize};

use byteorder::ByteOrder;
use std::io::Write;


impl Serialize for HeartbeatSubmessage {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> Deserialize<'de> for HeartbeatSubmessage {
    fn deserialize<B: ByteOrder>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> {
        todo!()
    }
}