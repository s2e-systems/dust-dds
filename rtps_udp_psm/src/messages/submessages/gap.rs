use super::SubmessageHeader;
use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

pub struct Gap {
    endianness_flag: SubmessageFlag,
    reader_id: <Self as rust_rtps_pim::messages::submessages::gap_submessage::Gap>::EntityId,
    writer_id: <Self as rust_rtps_pim::messages::submessages::gap_submessage::Gap>::EntityId,
    gap_start: <Self as rust_rtps_pim::messages::submessages::gap_submessage::Gap>::SequenceNumber,
    gap_list:
        <Self as rust_rtps_pim::messages::submessages::gap_submessage::Gap>::SequenceNumberSet,
}

impl Submessage for Gap {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::gap_submessage::Gap for Gap {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumber = submessage_elements::SequenceNumber;
    type SequenceNumberSet = submessage_elements::SequenceNumberSet;

    fn endianness_flag(&self) -> SubmessageFlag {
        self.endianness_flag
    }

    fn reader_id(&self) -> &Self::EntityId {
        &self.reader_id
    }

    fn writer_id(&self) -> &Self::EntityId {
        &self.writer_id
    }

    fn gap_start(&self) -> &Self::SequenceNumber {
        &self.gap_start
    }

    fn gap_list(&self) -> &Self::SequenceNumberSet {
        &self.gap_list
    }
}
