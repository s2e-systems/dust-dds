use crate::messages::{self, submessage_elements, Submessage};
pub trait InfoSource: Submessage {
    fn new(
        endianness_flag: <Self::PSM as messages::Types>::SubmessageFlag,
        protocol_version: submessage_elements::ProtocolVersion<Self::PSM>,
        vendor_id: submessage_elements::VendorId<Self::PSM>,
        guid_prefix: submessage_elements::GuidPrefix<Self::PSM>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self::PSM as messages::Types>::SubmessageFlag;
    fn protocol_version(&self) -> &submessage_elements::ProtocolVersion<Self::PSM>;
    fn vendor_id(&self) -> &submessage_elements::VendorId<Self::PSM>;
    fn guid_prefix(&self) -> &submessage_elements::GuidPrefix<Self::PSM>;
}
