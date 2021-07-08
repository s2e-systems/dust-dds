use rust_rtps_pim::messages::types::SubmessageFlag;

use crate::{
    submessage_elements::{GuidPrefixUdp, ProtocolVersionUdp, VendorIdUdp},
    submessage_header::SubmessageHeader,
};

#[derive(Debug, PartialEq)]
pub struct InfoSourceUdp;

impl<'a> rust_rtps_pim::messages::submessages::InfoSourceSubmessage for InfoSourceUdp {
    type ProtocolVersionSubmessageElementType = ProtocolVersionUdp;
    type VendorIdSubmessageElementType = VendorIdUdp;
    type GuidPrefixSubmessageElementType = GuidPrefixUdp;

    fn new(
        _endianness_flag: SubmessageFlag,
        _protocol_version: ProtocolVersionUdp,
        _vendor_id: VendorIdUdp,
        _guid_prefix: GuidPrefixUdp,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn protocol_version(&self) -> &ProtocolVersionUdp {
        todo!()
    }

    fn vendor_id(&self) -> &VendorIdUdp {
        todo!()
    }

    fn guid_prefix(&self) -> &GuidPrefixUdp {
        todo!()
    }
}

impl rust_rtps_pim::messages::Submessage for InfoSourceUdp {
    type RtpsSubmessageHeaderType = SubmessageHeader;
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }
}
