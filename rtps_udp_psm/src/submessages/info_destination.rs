use std::io::Write;
use byteorder::ByteOrder;
use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader};

use crate::submessage_elements::GuidPrefixUdp;

#[derive(Debug, PartialEq)]
pub struct InfoDestinationUdp;

impl crate::serialize::Serialize for InfoDestinationUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for InfoDestinationUdp {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::InfoDestinationSubmessageTrait for InfoDestinationUdp {
    type GuidPrefixSubmessageElementType = GuidPrefixUdp;
    fn new(_endianness_flag: SubmessageFlag, _guid_prefix: GuidPrefixUdp) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn guid_prefix(&self) -> &GuidPrefixUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoDestinationUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
