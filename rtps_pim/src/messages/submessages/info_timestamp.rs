use crate::messages::{self, submessage_elements, Submessage};

pub trait InfoTimestamp: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        invalidate_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        timestamp: submessage_elements::Timestamp<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn invalidate_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn timestamp(&self) -> &submessage_elements::Timestamp<Self::PSM>;
}
