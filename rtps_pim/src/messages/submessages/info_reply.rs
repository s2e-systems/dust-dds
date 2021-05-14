use crate::{messages::submessage_elements, PIM};

pub struct InfoReply<PSM: PIM> {
    pub endianness_flag: PSM::SubmessageFlag,
    pub multicast_flag: PSM::SubmessageFlag,
    pub unicast_locator_list: submessage_elements::LocatorList<PSM>,
    pub multicast_locator_list: submessage_elements::LocatorList<PSM>,
}
