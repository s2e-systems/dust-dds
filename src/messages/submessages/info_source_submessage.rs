use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;

#[derive(PartialEq, Debug)]
pub struct InfoSource {
    endianness_flag: SubmessageFlag,
    protocol_version: submessage_elements::ProtocolVersion,
    vendor_id: submessage_elements::VendorId,
    guid_prefix: submessage_elements::GuidPrefix,
}


impl Submessage for InfoSource {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        [e, X, X, X, X, X, X, X]
    }

    fn is_valid(&self) -> bool {
        true
    }
}