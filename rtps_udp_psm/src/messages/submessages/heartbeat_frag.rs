use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

pub struct HeartbeatFrag {
    endianness_flag: SubmessageFlag,
    reader_id: <Self as rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag>::EntityId,
    writer_id: <Self as rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag>::EntityId,
    writer_sn: <Self as rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag>::SequenceNumber,
    last_fragment_num: <Self as rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag>::FragmentNumber,
    count: <Self as rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag>::Count,
}

impl Submessage for HeartbeatFrag {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::heartbeat_frag_submessage::HeartbeatFrag
    for HeartbeatFrag
{
    type EntityId = submessage_elements::EntityId;
    type SequenceNumber = submessage_elements::SequenceNumber;
    type FragmentNumber = submessage_elements::FragmentNumber;
    type Count = submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn writer_sn(&self) -> &Self::SequenceNumber {
        &self.writer_sn
    }

    fn last_fragment_num(&self) -> &Self::FragmentNumber {
        &self.last_fragment_num
    }

    fn count(&self) -> &Self::Count {
        &self.count
    }
}
