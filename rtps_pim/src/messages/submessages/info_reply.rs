use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait InfoReply<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn multicast_flag(&self) -> PSM::SubmessageFlag;
    fn unicast_locator_list(&self) -> submessage_elements::LocatorList<PSM>;
    fn multicast_locator_list(&self) -> submessage_elements::LocatorList<PSM>;
}
