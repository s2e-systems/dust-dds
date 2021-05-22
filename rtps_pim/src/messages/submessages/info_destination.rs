use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait InfoDestination<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn guid_prefix(&self) -> submessage_elements::GuidPrefix<PSM>;
}
