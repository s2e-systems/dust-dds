use rust_rtps_pim::messages::submessages::NackFragSubmessage;

use crate::{deserialize::Deserialize, serialize::Serialize};

use byteorder::ByteOrder;
use std::io::Write;


impl<F> Serialize for NackFragSubmessage<F> {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de, F> Deserialize<'de> for NackFragSubmessage<F> {
    fn deserialize<B: ByteOrder>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> {
        todo!()
    }
}