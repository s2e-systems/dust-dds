use crate::{GuidPrefix, ProtocolVersion, RtpsUdpPsm, SubmessageFlag, VendorId};

use super::SubmessageHeader;

pub struct InfoSource;

impl<'a> rust_rtps_pim::messages::submessages::InfoSourceSubmessage<'a, RtpsUdpPsm> for InfoSource {
    fn new(
        _endianness_flag: SubmessageFlag,
        _protocol_version: ProtocolVersion,
        _vendor_id: VendorId,
        _guid_prefix: GuidPrefix,
    ) -> Self {
        todo!()
    }

    fn endianness_flag(&self) -> SubmessageFlag {
        todo!()
    }

    fn protocol_version(&self) -> &ProtocolVersion {
        todo!()
    }

    fn vendor_id(&self) -> &VendorId {
        todo!()
    }

    fn guid_prefix(&self) -> &GuidPrefix {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::Submessage<'a, RtpsUdpPsm> for InfoSource {
    fn submessage_header(&self) -> SubmessageHeader {
        todo!()
    }

    fn submessage_elements(
        &self,
    ) -> &[rust_rtps_pim::messages::submessage_elements::SubmessageElements<'a, RtpsUdpPsm>] {
        todo!()
    }
}
