use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{Count, EntityId, FragmentNumberSet, SequenceNumber},
    submessage_header::SubmessageHeader,
};
use rust_rtps_pim::messages::types::SubmessageFlag;

#[derive(Debug, PartialEq)]
pub struct NackFrag;

impl rust_rtps_pim::messages::submessages::NackFragSubmessage<RtpsUdpPsm> for NackFrag {
    fn new(
        _endianness_flag: SubmessageFlag,
        _reader_id: EntityId,
        _writer_id: EntityId,
        _writer_sn: SequenceNumber,
        _fragment_number_state: FragmentNumberSet,
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

    fn fragment_number_state(&self) -> &FragmentNumberSet {
        todo!()
    }

    fn count(&self) -> &Count {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for NackFrag {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
