use crate::messages::{submessage_elements, types::SubmessageFlag};
use super::SubmessageHeader;

struct AckNack {
    pub endianness_flag: SubmessageFlag,
    pub final_flag: SubmessageFlag,
    pub reader_id: submessage_elements::EntityId,
    pub writer_id: submessage_elements::EntityId,
    pub reader_sn_state: submessage_elements::SequenceNumberSet,
    pub count: submessage_elements::Count,
}

impl rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack for AckNack {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumberSet = submessage_elements::SequenceNumberSet;
    type Count = submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn final_flag(&self) -> SubmessageFlag {
        self.final_flag
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn reader_sn_state(&self) -> &Self::SequenceNumberSet {
        &self.reader_sn_state
    }

    fn count(&self) -> &Self::Count {
        &self.count
    }
}

impl rust_rtps_pim::messages::submessages::Submessage for AckNack {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }
}
