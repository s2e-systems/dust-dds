use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait InfoReply: Submessage {
    type LocatorList: submessage_elements::LocatorList;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn multicast_flag(&self) -> SubmessageFlag;
    fn unicast_locator_list(&self) -> &Self::LocatorList;
    fn multicast_locator_list(&self) -> &Self::LocatorList;
}
