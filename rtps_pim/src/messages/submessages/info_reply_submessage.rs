use super::{submessage_elements, Submessage};
use crate::types;

pub trait InfoReply: Submessage {
    type Locator: types::Locator;
    type LocatorList: IntoIterator<Item = Self::Locator>;
    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        multicast_flag: <Self as Submessage>::SubmessageFlag,
        unicast_locator_list: submessage_elements::LocatorList<Self::Locator, Self::LocatorList>,
        multicast_locator_list: submessage_elements::LocatorList<Self::Locator, Self::LocatorList>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn multicast_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn unicast_locator_list(
        &self,
    ) -> &submessage_elements::LocatorList<Self::Locator, Self::LocatorList>;
    fn multicast_locator_list(
        &self,
    ) -> &submessage_elements::LocatorList<Self::Locator, Self::LocatorList>;
}
