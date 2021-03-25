use crate::{messages, types};

pub struct Header<
    ProtocolId: messages::types::ProtocolId,
    ProtocolVersion: types::ProtocolVersion,
    VendorId: types::VendorId,
    GuidPrefix: types::GuidPrefix,
> {
    pub protocol: ProtocolId,
    pub version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}
