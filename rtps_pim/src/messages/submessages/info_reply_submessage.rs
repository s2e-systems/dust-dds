use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait InfoReply: Submessage {
    type LocatorList: submessage_elements::LocatorList;
    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        multicast_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        unicast_locator_list: Self::LocatorList,
        multicast_locator_list: Self::LocatorList,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn multicast_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn unicast_locator_list(&self) -> &Self::LocatorList;
    fn multicast_locator_list(&self) -> &Self::LocatorList;
}
