use crate::{
    messages::{self, submessage_elements, Submessage},
    structure,
};

pub trait InfoTimestamp<PSM: structure::Types + messages::Types>: Submessage<PSM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn invalidate_flag(&self) -> PSM::SubmessageFlag;
    fn timestamp(&self) -> submessage_elements::Timestamp<PSM>;
}
