use std::io::Write;
use byteorder::ByteOrder;
use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader};

use crate::submessage_elements::{CountUdp, EntityIdUdp, FragmentNumberUdp, SequenceNumberUdp};

#[derive(Debug, PartialEq)]
pub struct HeartbeatFragUdp;

impl crate::serialize::Serialize for HeartbeatFragUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for HeartbeatFragUdp {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::HeartbeatFragSubmessageTrait for HeartbeatFragUdp {
    type EntityIdSubmessageElementType = EntityIdUdp;
    type SequenceNumberSubmessageElementType = SequenceNumberUdp;
    type FragmentNumberSubmessageElementType = FragmentNumberUdp;
    type CountSubmessageElementType = CountUdp;

    fn new(
        _endianness_flag: SubmessageFlag,
        _reader_id: EntityIdUdp,
        _writer_id: EntityIdUdp,
        _writer_sn: SequenceNumberUdp,
        _last_fragment_num: FragmentNumberUdp,
        _count: CountUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &EntityIdUdp {
        todo!()
    }

    fn writer_id(&self) -> &EntityIdUdp {
        todo!()
    }

    fn writer_sn(&self) -> &SequenceNumberUdp {
        todo!()
    }

    fn last_fragment_num(&self) -> &FragmentNumberUdp {
        todo!()
    }

    fn count(&self) -> &CountUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for HeartbeatFragUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
