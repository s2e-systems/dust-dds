use crate::RtpsUdpPsm;

pub struct HeartbeatFrag;

impl rust_rtps_pim::messages::submessages::HeartbeatFrag<RtpsUdpPsm> for HeartbeatFrag {
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

    fn last_fragment_num(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::FragmentNumber<RtpsUdpPsm> {
        todo!()
    }

    fn count(&self) -> rust_rtps_pim::messages::submessage_elements::Count<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for HeartbeatFrag {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
