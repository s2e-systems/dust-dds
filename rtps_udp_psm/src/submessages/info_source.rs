use std::io::Write;
use byteorder::ByteOrder;
use rust_rtps_pim::messages::{types::SubmessageFlag, RtpsSubmessageHeader};

use crate::submessage_elements::{GuidPrefixUdp, ProtocolVersionUdp, VendorIdUdp};

#[derive(Debug, PartialEq)]
pub struct InfoSourceUdp;

impl crate::serialize::Serialize for InfoSourceUdp {
    fn serialize<W: Write, B: ByteOrder>(&self, mut _writer: W) -> crate::serialize::Result {
        todo!()
    }
}
impl<'de> crate::deserialize::Deserialize<'de> for InfoSourceUdp {
    fn deserialize<B>(_buf: &mut &'de[u8]) -> crate::deserialize::Result<Self> where B: ByteOrder {
        todo!()
    }
}

impl<'a> rust_rtps_pim::messages::submessages::InfoSourceSubmessageTrait for InfoSourceUdp {
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
    fn submessage_header(&self) -> RtpsSubmessageHeader {
        todo!()
    }
}
