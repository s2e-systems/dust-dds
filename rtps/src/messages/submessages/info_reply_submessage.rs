use super::submessage_elements;
use super::SubmessageFlag;
use super::{Submessage, SubmessageHeader};

use crate::messages::types::constants;

#[derive(PartialEq, Debug)]
pub struct InfoReply {
    pub endianness_flag: SubmessageFlag,
    pub multicast_flag: SubmessageFlag,
    pub unicast_locator_list: submessage_elements::LocatorList,
    pub multicast_locator_list: submessage_elements::LocatorList,
}

impl Submessage for InfoReply {
    fn submessage_header(&self) -> SubmessageHeader {
        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let m = self.multicast_flag;
        let flags = [e, m, X, X, X, X, X, X];

        SubmessageHeader::new(constants::SUBMESSAGE_KIND_INFO_REPLY, flags, 0)
    }

    fn is_valid(&self) -> bool {
        true
    }
}

impl serde::Serialize for InfoReply {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}
