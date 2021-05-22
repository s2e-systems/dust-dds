use rust_rtps_pim::messages::Submessage;

use crate::RtpsUdpPsm;

pub struct AckNack {}

impl rust_rtps_pim::messages::submessages::AckNack<RtpsUdpPsm> for AckNack {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn final_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn reader_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn writer_id(&self) -> rust_rtps_pim::messages::submessage_elements::EntityId<RtpsUdpPsm> {
        todo!()
    }

    fn reader_sn_state(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::SequenceNumberSet<RtpsUdpPsm> {
        todo!()
    }

    fn count(&self) -> rust_rtps_pim::messages::submessage_elements::Count<RtpsUdpPsm> {
        todo!()
    }
}

impl Submessage<RtpsUdpPsm> for AckNack {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
