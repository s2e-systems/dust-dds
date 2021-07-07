use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{Count, EntityId, FragmentNumber, SequenceNumber},
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct HeartbeatFrag;

impl<'a> rust_rtps_pim::messages::submessages::HeartbeatFragSubmessage<RtpsUdpPsm<'a>> for HeartbeatFrag {
    fn new(
        _endianness_flag: SubmessageFlag,
        _reader_id: EntityId,
        _writer_id: EntityId,
        _writer_sn: SequenceNumber,
        _last_fragment_num: FragmentNumber,
        _count: Count,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &EntityId {
        todo!()
    }

    fn writer_id(&self) -> &EntityId {
        todo!()
    }

    fn writer_sn(&self) -> &SequenceNumber {
        todo!()
    }

    fn last_fragment_num(&self) -> &FragmentNumber {
        todo!()
    }

    fn count(&self) -> &Count {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for HeartbeatFrag {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
