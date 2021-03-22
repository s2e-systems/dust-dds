use crate::{messages, types};

pub struct Header<
    ProtocolId: messages::types::ProtocolId,
    ProtocolVersion: types::ProtocolVersion,
    VendorId: types::VendorId,
    GuidPrefix: types::GuidPrefix,
> {
    pub protocol_id: ProtocolId,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub guid_prefix: GuidPrefix,
}
