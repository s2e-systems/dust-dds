use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{Count, EntityId, SequenceNumberSet},
};
use rust_rtps_pim::messages::{types::SubmessageFlag, Submessage};
use super::header::SubmessageHeader;

#[derive(Debug, PartialEq)]
pub struct AckNack {}

impl rust_rtps_pim::messages::submessages::AckNackSubmessage<RtpsUdpPsm> for AckNack {
    fn new(
        _endianness_flag: SubmessageFlag,
        _final_flag: SubmessageFlag,
        _reader_id: EntityId,
        _writer_id: EntityId,
        _reader_sn_state: SequenceNumberSet,
        _count: Count,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &EntityId {
        todo!()
    }

    fn writer_id(&self) -> &EntityId {
        todo!()
    }

    fn reader_sn_state(&self) -> &SequenceNumberSet {
        todo!()
    }

    fn count(&self) -> &Count {
        todo!()
    }
}

impl Submessage<RtpsUdpPsm> for AckNack {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
