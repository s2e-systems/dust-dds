use crate::types::{GuidPrefix, ProtocolVersion, VendorId};
use super::types::ProtocolId;

pub trait Header {
    fn protocol(&self) -> ProtocolId;
    fn version(&self) -> ProtocolVersion;
    fn vendor_id(&self) -> VendorId;
    fn guid_prefix(&self) -> GuidPrefix;
}
