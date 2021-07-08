use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    submessage_elements::{CountUdp, EntityIdUdp, FragmentNumberUdp, SequenceNumberUdp},
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct HeartbeatFragUdp;

impl<'a> rust_rtps_pim::messages::submessages::HeartbeatFragSubmessage for HeartbeatFragUdp {
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
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
