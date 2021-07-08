use crate::{psm::RtpsUdpPsm, submessage_elements::{CountUdp, EntityIdUdp, SequenceNumberSetUdp}, submessage_header::SubmessageHeader};
use rust_rtps_pim::messages::{types::SubmessageFlag, Submessage};

#[derive(Debug, PartialEq)]
pub struct AckNackUdp {}

impl<'a> rust_rtps_pim::messages::submessages::AckNackSubmessage<RtpsUdpPsm<'a>> for AckNackUdp {
    fn new(
        _endianness_flag: SubmessageFlag,
        _final_flag: SubmessageFlag,
        _reader_id: EntityIdUdp,
        _writer_id: EntityIdUdp,
        _reader_sn_state: SequenceNumberSetUdp,
        _count: CountUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> &EntityIdUdp {
        todo!()
    }

    fn writer_id(&self) -> &EntityIdUdp {
        todo!()
    }

    fn reader_sn_state(&self) -> &SequenceNumberSetUdp {
        todo!()
    }

    fn count(&self) -> &CountUdp {
        todo!()
    }
}

impl Submessage for AckNackUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
