use crate::RtpsUdpPsm;

pub struct InfoSource;

impl rust_rtps_pim::messages::submessages::InfoSource<RtpsUdpPsm> for InfoSource {
    fn endianness_flag(&self) -> <RtpsUdpPsm as rust_rtps_pim::messages::Types>::SubmessageFlag {
        todo!()
    }

    fn protocol_version(
        &self,
    ) -> rust_rtps_pim::messages::submessage_elements::ProtocolVersion<RtpsUdpPsm> {
        todo!()
    }

    fn vendor_id(&self) -> rust_rtps_pim::messages::submessage_elements::VendorId<RtpsUdpPsm> {
        todo!()
    }

    fn guid_prefix(&self) -> rust_rtps_pim::messages::submessage_elements::GuidPrefix<RtpsUdpPsm> {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage<RtpsUdpPsm> for InfoSource {
    fn submessage_header(&self) -> rust_rtps_pim::messages::SubmessageHeader<RtpsUdpPsm> {
        todo!()
    }
}
