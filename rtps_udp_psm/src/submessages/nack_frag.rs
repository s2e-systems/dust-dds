use crate::RtpsUdpPsm;

pub struct NackFrag;

impl rust_rtps_pim::messages::submessages::NackFrag<RtpsUdpPsm> for NackFrag {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_sn(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::SequenceNumber<RtpsUdpPsm> {
        todo!()
    }

    fn fragment_number_state(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::FragmentNumberSet<RtpsUdpPsm> {
        todo!()
    }

    fn count(&self) -> rust_rtps_pim::messages::submessage_elements::Count<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for NackFrag {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
