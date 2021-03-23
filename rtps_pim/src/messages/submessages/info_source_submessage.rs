use super::{submessage_elements, Submessage, SubmessageHeader};

pub trait InfoSource: Submessage {
    type ProtocolVersion: submessage_elements::ProtocolVersion;
    type VendorId: submessage_elements::VendorId;
    type GuidPrefix: submessage_elements::GuidPrefix;

    fn new(
        endianness_flag: <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag,
        protocol_version: Self::ProtocolVersion,
        vendor_id: Self::VendorId,
        guid_prefix: Self::GuidPrefix,
    ) -> Self;

    fn endianness_flag(
        &self,
    ) -> <<Self as Submessage>::SubmessageHeader as SubmessageHeader>::SubmessageFlag;
    fn protocol_version(&self) -> &Self::ProtocolVersion;
    fn vendor_id(&self) -> &Self::VendorId;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}
