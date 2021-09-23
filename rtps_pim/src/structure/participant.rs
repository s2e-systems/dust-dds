use super::{
    types::{ProtocolVersion, VendorId},
    RtpsEntity,
};

pub struct RtpsParticipant<L> {
    pub entity: RtpsEntity,
    pub protocol_version: ProtocolVersion,
    pub vendor_id: VendorId,
    pub default_unicast_locator_list: L,
    pub default_multicast_locator_list: L,
}
