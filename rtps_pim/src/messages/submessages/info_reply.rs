use crate::{messages::submessage_elements, PIM};

pub trait InfoReply<PSM: PIM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn multicast_flag(&self) -> PSM::SubmessageFlag;
    fn unicast_locator_list(&self) -> submessage_elements::LocatorList<PSM>;
    fn multicast_locator_list(&self) -> submessage_elements::LocatorList<PSM>;
}
