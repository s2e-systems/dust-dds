use super::{SubmessageKind, SubmessageFlag, };
use super::{Submessage, SubmessageHeader, };
use super::submessage_elements;

#[derive(PartialEq, Debug)]
pub struct InfoDestination {
    endianness_flag: SubmessageFlag,
    guid_prefix: submessage_elements::GuidPrefix,
}

impl Submessage for InfoDestination {
    fn submessage_flags(&self) -> [SubmessageFlag; 8] {
        const X : SubmessageFlag = false;
        let e = self.endianness_flag; // Indicates endianness.
        [e, X, X, X, X, X, X, X]
    }

    fn is_valid(&self) -> bool {
        true    
    }   
}