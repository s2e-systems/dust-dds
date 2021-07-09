use crate::submessage_elements::{CountUdp, EntityIdUdp, SequenceNumberSetUdp};
use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader, Submessage};

#[derive(Debug, PartialEq)]
pub struct AckNackUdp {}

impl<'a> rust_rtps_pim::messages::submessages::AckNackSubmessage for AckNackUdp {
    type EntityIdSubmessageElementType = EntityIdUdp;
    type SequenceNumberSetSubmessageElementType = SequenceNumberSetUdp;
    type CountSubmessageElementType = CountUdp;

    fn new(
        _endianness_flag: SubmessageFlag,
        _final_flag: SubmessageFlag,
        _reader_id: EntityIdUdp,
        _writer_id: EntityIdUdp,
        _reader_sn_state: SequenceNumberSetUdp,
        _count: CountUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &EntityIdUdp {
        todo!()
    }

    fn writer_id(&self) -> &EntityIdUdp {
        todo!()
    }

    fn reader_sn_state(&self) -> &SequenceNumberSetUdp {
        todo!()
    }

    fn count(&self) -> &CountUdp {
        todo!()
    }
}

impl Submessage for AckNackUdp {
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
