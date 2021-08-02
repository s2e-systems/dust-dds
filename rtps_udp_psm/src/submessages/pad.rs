use std::io::Write;
use byteorder::ByteOrder;
use rust_rtps_pim::messages::RtpsSubmessageHeader;

#[derive(Debug, PartialEq)]
pub struct PadUdp;

impl crate::serialize::Serialize for PadUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for PadUdp {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::PadSubmessage for PadUdp {}

impl rust_rtps_pim::messages::Submessage for PadUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
