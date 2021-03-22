use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait InfoDestination: Submessage {
    type GuidPrefix: submessage_elements::GuidPrefix;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}
