use crate::RtpsUdpPsm;

pub struct Gap;

impl rust_rtps_pim::messages::submessages::Gap<RtpsUdpPsm> for Gap {
    fn new(
        _endianness_flag: <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag,
        _reader_id: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::EntityId,
        _writer_id: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::EntityId,
        _gap_start: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumber,
        _gap_list: <RtpsUdpPsm as rust_rtps_pim::structure::Types>::SequenceNumberVector,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn gap_start(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::SequenceNumber<RtpsUdpPsm> {
        todo!()
    }

    fn gap_list(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::SequenceNumberSet<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for Gap {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
