use super::submessage_elements;
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag};

use crate::messages::types::constants;

#[derive(PartialEq, Debug)]
pub struct InfoDestination {
    pub endianness_flag: SubmessageFlag,
    pub guid_prefix: submessage_elements::GuidPrefix,
}

impl Submessage for InfoDestination {
    fn submessage_header(&self) -> SubmessageHeader {

        const X: SubmessageFlag = false;
        let e = self.endianness_flag; // Indicates endianness.
        let flags = [e, X, X, X, X, X, X, X];

        SubmessageHeader::new(constants::SUBMESSAGE_KIND_INFO_DESTINATION, flags, 0)
    }

    fn is_valid(&self) -> bool {
        true
    }
}

impl serde::Serialize for InfoDestination {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
