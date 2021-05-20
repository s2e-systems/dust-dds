use crate::{messages::submessage_elements, PIM};

pub trait InfoTimestamp<PSM: PIM> {
    fn endianness_flag(&self) -> PSM::SubmessageFlag;
    fn invalidate_flag(&self) -> PSM::SubmessageFlag;
    fn timestamp(&self) -> submessage_elements::Timestamp<PSM>;
}
