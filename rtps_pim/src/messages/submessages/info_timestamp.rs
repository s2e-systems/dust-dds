use super::{submessage_elements, Submessage};
use crate::{messages};

pub trait InfoTimestamp: Submessage {
    type Time: messages::types::Time;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        invalidate_flag: <Self as Submessage>::SubmessageFlag,
        timestamp: submessage_elements::Timestamp<Self::Time>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn invalidate_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn timestamp(&self) -> &submessage_elements::Timestamp<Self::Time>;
}
