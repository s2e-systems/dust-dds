use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;

#[derive(PartialEq, Debug)]
pub struct InfoDestination {
    endianness_flag: SubmessageFlag,
    guid_prefix: submessage_elements::GuidPrefix,
}

impl Submessage for InfoDestination {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::InfoDestination;

        const X : SubmessageFlag = false;
        let e = self.endianness_flag; // Indicates endianness.
        let flags = [e, X, X, X, X, X, X, X];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
    }

    fn is_valid(&self) -> bool {
        true    
    }   
}