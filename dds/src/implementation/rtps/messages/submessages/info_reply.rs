use crate::implementation::rtps::messages::{
    overall_structure::SubmessageHeaderRead, submessage_elements::LocatorList,
    types::SubmessageFlag, RtpsMap, SubmessageHeader,
};

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessageRead<'a> {
    data: &'a [u8],
}

impl SubmessageHeader for InfoReplySubmessageRead<'_> {
    fn submessage_header(&self) -> SubmessageHeaderRead {
        SubmessageHeaderRead::new(self.data)
    }
}

impl<'a> InfoReplySubmessageRead<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    pub fn multicast_flag(&self) -> bool {
        self.submessage_header().flags()[1]
    }

    pub fn unicast_locator_list(&self) -> LocatorList {
        self.map(&self.data[4..])
    }

    pub fn multicast_locator_list(&self) -> LocatorList {
        if self.multicast_flag() {
            let num_locators: u32 = self.map(&self.data[4..]);
            let octets_to_multicat_loctor_list = num_locators as usize * 24 + 8;
            self.map(&self.data[octets_to_multicat_loctor_list..])
        } else {
            LocatorList::new(vec![])
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InfoReplySubmessageWrite {
    pub endianness_flag: SubmessageFlag,
    pub multicast_flag: SubmessageFlag,
    pub unicast_locator_list: LocatorList,
    pub multicast_locator_list: LocatorList,
}
