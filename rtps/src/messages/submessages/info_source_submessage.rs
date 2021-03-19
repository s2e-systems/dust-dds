use super::submessage_elements;
use super::Submessage;
use super::SubmessageFlag;

pub trait InfoSource: Submessage {
    type ProtocolVersion: submessage_elements::ProtocolVersion;
    type VendorId: submessage_elements::VendorId;
    type GuidPrefix: submessage_elements::GuidPrefix;

    fn endianness_flag(&self) -> SubmessageFlag;
    fn protocol_version(&self) -> &Self::ProtocolVersion;
    fn vendor_id(&self) -> &Self::VendorId;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}
