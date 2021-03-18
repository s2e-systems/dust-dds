use super::submessage_elements;
use super::{Submessage, SubmessageHeader};
use super::{SubmessageFlag, SubmessageKind};

#[derive(PartialEq, Debug)]
pub struct InfoReply {
    pub endianness_flag: SubmessageFlag,
    pub multicast_flag: SubmessageFlag,
    pub unicast_locator_list: submessage_elements::LocatorList,
    pub multicast_locator_list: submessage_elements::LocatorList,
}

impl Submessage for InfoReply {
    fn submessage_header(&self, octets_to_next_header: u16) -> SubmessageHeader {
        let submessage_id = SubmessageKind::InfoReply;

        const X: SubmessageFlag = false;
        let e = self.endianness_flag;
        let m = self.multicast_flag;
        let flags = [e, m, X, X, X, X, X, X];

        SubmessageHeader::new(submessage_id, flags, octets_to_next_header)
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
