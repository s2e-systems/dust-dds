use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    psm::RtpsUdpPsm,
    submessage_elements::{GuidPrefix, ProtocolVersionC, VendorId},
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct InfoSource;

impl<'a> rust_rtps_pim::messages::submessages::InfoSourceSubmessage<RtpsUdpPsm> for InfoSource {
    fn new(
        _endianness_flag: SubmessageFlag,
        _protocol_version: ProtocolVersionC,
        _vendor_id: VendorId,
        _guid_prefix: GuidPrefix,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn protocol_version(&self) -> &ProtocolVersionC {
        todo!()
    }

    fn vendor_id(&self) -> &VendorId {
        todo!()
    }

    fn guid_prefix(&self) -> &GuidPrefix {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoSource {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
