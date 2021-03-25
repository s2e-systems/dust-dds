use super::{submessage_elements, Submessage};
use crate::types;

pub trait InfoSource: Submessage {
    type ProtocolVersion: types::ProtocolVersion;
    type VendorId: types::VendorId;
    type GuidPrefix: types::GuidPrefix;

    fn new(
        endianness_flag: <Self as Submessage>::SubmessageFlag,
        protocol_version: submessage_elements::ProtocolVersion<Self::ProtocolVersion>,
        vendor_id: submessage_elements::VendorId<Self::VendorId>,
        guid_prefix: submessage_elements::GuidPrefix<Self::GuidPrefix>,
    ) -> Self;

    fn endianness_flag(&self) -> <Self as Submessage>::SubmessageFlag;
    fn protocol_version(&self) -> &submessage_elements::ProtocolVersion<Self::ProtocolVersion>;
    fn vendor_id(&self) -> &submessage_elements::VendorId<Self::VendorId>;
    fn guid_prefix(&self) -> &submessage_elements::GuidPrefix<Self::GuidPrefix>;
}
