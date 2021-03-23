use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait InfoDestination: Submessage {
    type GuidPrefix: submessage_elements::GuidPrefix;

    fn new(
        endianness_flag:<<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        guid_prefix: Self::GuidPrefix,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}
