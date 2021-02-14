use crate::{
    behavior::types::Duration,
    messages::types::Count,
    types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
};

use super::types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId};

pub struct ParticipantProxy {
    pub domain_id: DomainId,
    pub domain_tag: String,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub metatraffic_unicast_locator_list: Vec<Locator>,
    pub metatraffic_multicast_locator_list: Vec<Locator>,
    pub default_unicast_locator_list: Vec<Locator>,
    pub default_multicast_locator_list: Vec<Locator>,
    pub available_builtin_endpoints: BuiltinEndpointSet,
    pub lease_duration: Duration,
    pub manual_liveliness_count: Count,
    pub builtin_endpoint_qos: BuiltinEndpointQos,
}
