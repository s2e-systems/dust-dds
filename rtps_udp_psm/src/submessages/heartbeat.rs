use crate::RtpsUdpPsm;

pub struct Heartbeat;

impl rust_rtps_pim::messages::submessages::Heartbeat<RtpsUdpPsm> for Heartbeat {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn liveliness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn first_sn(&self) -> rust_rtps_pim::messages::submessage_elements::SequenceNumber<RtpsUdpPsm> {
        todo!()
    }

    fn last_sn(&self) -> rust_rtps_pim::messages::submessage_elements::SequenceNumber<RtpsUdpPsm> {
        todo!()
    }

    fn count(&self) -> rust_rtps_pim::messages::submessage_elements::Count<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for Heartbeat {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
