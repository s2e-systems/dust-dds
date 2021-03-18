use crate::{
    messages::types::Count,
    types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
};

use super::types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId};

pub struct ParticipantProxy {
    pub domain_id: DomainId,
    pub domain_tag: &'static str,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub metatraffic_unicast_locator_list: &'static [Locator],
    pub metatraffic_multicast_locator_list: &'static [Locator],
    pub default_unicast_locator_list: &'static [Locator],
    pub default_multicast_locator_list: &'static [Locator],
    pub available_builtin_endpoints: BuiltinEndpointSet,
    pub manual_liveliness_count: Count,
    pub builtin_endpoint_qos: BuiltinEndpointQos,
}
