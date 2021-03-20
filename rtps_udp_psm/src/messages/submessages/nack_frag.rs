use crate::messages::submessage_elements;
use rust_rtps_pim::messages::{submessages::Submessage, types::SubmessageFlag};

use super::SubmessageHeader;

pub struct NackFrag {
    endianness_flag: SubmessageFlag,
    reader_id: <Self as rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag>::EntityId,
    writer_id: <Self as rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag>::EntityId,
    writer_sn: <Self as rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag>::SequenceNumber,
    fragment_number_state: <Self as rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag>::FragmentNumberSet,
    count: <Self as rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag>::Count,
}

impl Submessage for NackFrag {
    type SubmessageHeader = SubmessageHeader;

    fn submessage_header(&self) -> Self::SubmessageHeader {
        todo!()
    }

    fn is_valid(&self) -> bool {
        todo!()
    }
}

impl rust_rtps_pim::messages::submessages::nack_frag_submessage::NackFrag for NackFrag {
    type EntityId = submessage_elements::EntityId;
    type SequenceNumber = submessage_elements::SequenceNumber;
    type FragmentNumberSet = submessage_elements::FragmentNumberSet;
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

    fn fragment_number_state(&self) -> &Self::FragmentNumberSet {
        &self.fragment_number_state
    }

    fn count(&self) -> &Self::Count {
        &self.count
    }
}
