use crate::messages::{submessage_elements, types::SubmessageFlag};
use serde::Serialize;

use super::SubmessageHeader;

#[derive(Serialize)]
struct AckNack {
    submessage_header: SubmessageHeader,
    reader_id: submessage_elements::EntityId,
    writer_id: submessage_elements::EntityId,
    reader_sn_state: submessage_elements::SequenceNumberSet,
    count: submessage_elements::Count,
}

impl rust_rtps_pim::messages::submessages::ack_nack_submessage::AckNack for AckNack {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumberSet = submessage_elements::SequenceNumberSet;
    type Count = submessage_elements::Count;

    fn endianness_flag(&self) -> SubmessageFlag {
        rust_rtps_pim::messages::submessages::SubmessageHeader::flags(&self.submessage_header)[0]
    }

    fn final_flag(&self) -> SubmessageFlag {
        rust_rtps_pim::messages::submessages::SubmessageHeader::flags(&self.submessage_header)[1]
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
        self.submessage_header
    }
}
