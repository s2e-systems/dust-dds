use crate::messages::{self, submessage_elements, Submessage};

pub trait InfoReply: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        multicast_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        unicast_locator_list: submessage_elements::LocatorList<Self::PSM>,
        multicast_locator_list: submessage_elements::LocatorList<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn multicast_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn unicast_locator_list(&self) -> &submessage_elements::LocatorList<Self::PSM>;
    fn multicast_locator_list(&self) -> &submessage_elements::LocatorList<Self::PSM>;
}
