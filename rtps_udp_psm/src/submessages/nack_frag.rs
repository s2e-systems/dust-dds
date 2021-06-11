use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{Count, EntityId, FragmentNumberSet, RtpsUdpPsm, SequenceNumber};

use super::SubmessageHeader;

pub struct NackFrag;

impl<'a> rust_rtps_pim::messages::submessages::NackFragSubmessage<'a, RtpsUdpPsm> for NackFrag {
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

impl<'a> rust_rtps_pim::messages::Submessage<'a, RtpsUdpPsm> for NackFrag {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }

    fn submessage_elements(
        &self,
    ) -> &[rust_rtps_pim::messages::submessage_elements::SubmessageElements<'a, RtpsUdpPsm>] {
        todo!()
    }
}
