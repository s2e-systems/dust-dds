use std::io::Write;
use byteorder::ByteOrder;
use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader};

use crate::submessage_elements::TimeUdp;

#[derive(Debug, PartialEq)]
pub struct InfoTimestampUdp;

impl crate::serialize::Serialize for InfoTimestampUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for InfoTimestampUdp {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::InfoTimestampSubmessage for InfoTimestampUdp {
    type TimestampSubmessageElementType = TimeUdp;

    fn new(
        _endianness_flag: SubmessageFlag,
        _invalidate_flag: SubmessageFlag,
        _timestamp: TimeUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn invalidate_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn timestamp(&self) -> &TimeUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoTimestampUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
