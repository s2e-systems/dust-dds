use crate::{
    submessage_elements::{CountUdp, EntityIdUdp, FragmentNumberSetUdp, SequenceNumberUdp},
    submessage_header::SubmessageHeader,
};
use rust_rtps_pim::messages::types::SubmessageFlag;

#[derive(Debug, PartialEq)]
pub struct NackFragUdp;

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
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
