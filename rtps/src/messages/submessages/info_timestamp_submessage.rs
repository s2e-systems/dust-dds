use super::submessage_elements;
use super::{Submessage, SubmessageFlag};

pub trait InfoTimestamp: Submessage {
    type Timestamp: submessage_elements::Timestamp;
    fn endianness_flag(&self) -> SubmessageFlag;
    fn invalidate_flag(&self) -> SubmessageFlag;
    fn timestamp(&self) -> &Self::Timestamp;
}
