use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait InfoTimestamp: Submessage {
    type Timestamp: submessage_elements::Timestamp;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        invalidate_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        timestamp: Self::Timestamp,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn invalidate_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn timestamp(&self) -> &Self::Timestamp;
}
