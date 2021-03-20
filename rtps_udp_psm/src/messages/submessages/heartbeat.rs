use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

pub struct Heartbeat {
    endianness_flag: SubmessageFlag,
    final_flag: SubmessageFlag,
    liveliness_flag: SubmessageFlag,
    reader_id: <Self as rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat>::EntityId,
    writer_id: <Self as rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat>::EntityId,
    first_sn: <Self as rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat>::SequenceNumber,
    last_sn: <Self as rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat>::SequenceNumber,
    count: <Self as rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat>::Count,
}

impl Submessage for Heartbeat {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::heartbeat_submessage::Heartbeat for Heartbeat {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumber = submessage_elements::SequenceNumber;
    type Count = submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn final_flag(&self) -> SubmessageFlag {
        self.final_flag
    }

    fn liveliness_flag(&self) -> SubmessageFlag {
        self.liveliness_flag
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn first_sn(&self) -> &Self::SequenceNumber {
        &self.first_sn
    }

    fn last_sn(&self) -> &Self::SequenceNumber {
        &self.last_sn
    }

    fn count(&self) -> &Self::Count {
        &self.count
    }
}
