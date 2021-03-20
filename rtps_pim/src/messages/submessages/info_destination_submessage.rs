use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait InfoDestination: Submessage {
    type GuidPrefix: submessage_elements::GuidPrefix;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}
