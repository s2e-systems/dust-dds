use std::io::Write;
use byteorder::ByteOrder;
use crate::submessage_elements::{CountUdp, EntityIdUdp, FragmentNumberSetUdp, SequenceNumberUdp};
use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader};

#[derive(Debug, PartialEq)]
pub struct NackFragUdp;

impl crate::serialize::Serialize for NackFragUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for NackFragUdp {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::NackFragSubmessage for NackFragUdp {
    type EntityIdSubmessageElementType = EntityIdUdp;
    type SequenceNumberSubmessageElementType = SequenceNumberUdp;
    type FragmentNumberSetSubmessageElementType = FragmentNumberSetUdp;
    type CountSubmessageElementType = CountUdp;

    fn new(
        _endianness_flag: SubmessageFlag,
        _reader_id: EntityIdUdp,
        _writer_id: EntityIdUdp,
        _writer_sn: SequenceNumberUdp,
        _fragment_number_state: FragmentNumberSetUdp,
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

    fn fragment_number_state(&self) -> &FragmentNumberSetUdp {
        todo!()
    }

    fn count(&self) -> &CountUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for NackFragUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
