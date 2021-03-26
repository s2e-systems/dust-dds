use crate::messages::{self, submessage_elements, Submessage};

pub trait InfoDestination: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        guid_prefix: submessage_elements::GuidPrefix<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn guid_prefix(&self) -> &submessage_elements::GuidPrefix<Self::PSM>;
}
