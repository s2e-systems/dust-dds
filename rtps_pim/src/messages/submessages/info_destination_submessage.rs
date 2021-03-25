use super::{submessage_elements, Submessage};
use crate::types;

pub trait InfoDestination: Submessage {
    type GuidPrefix: types::GuidPrefix;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        guid_prefix: submessage_elements::GuidPrefix<Self::GuidPrefix>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn guid_prefix(&self) -> &submessage_elements::GuidPrefix<Self::GuidPrefix>;
}
