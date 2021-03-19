use super::types::ProtocolId;
use crate::types::{GuidPrefix, ProtocolVersion, VendorId};

pub trait Header {
    type ProtocolId: ProtocolId;
    type ProtocolVersion: ProtocolVersion;
    type VendorId: VendorId;
    type GuidPrefix: GuidPrefix;

    fn protocol(&self) -> &Self::ProtocolId;
    fn version(&self) -> &Self::ProtocolVersion;
    fn vendor_id(&self) -> &Self::VendorId;
    fn guid_prefix(&self) -> &Self::GuidPrefix;
}
