use crate::{behavior::types::Duration, discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId}, messages::types::Count, structure::types::{GuidPrefix, ProtocolVersion, VendorId}};

pub struct SpdpDiscoveredParticipantData<L> {
    pub domain_id: DomainId,
    pub domain_tag: &'static str,
    pub protocol_version: ProtocolVersion,
    pub guid_prefix: GuidPrefix,
    pub vendor_id: VendorId,
    pub expects_inline_qos: bool,
    pub metatraffic_unicast_locator_list: L,
    pub metatraffic_multicast_locator_list: L,
    pub default_unicast_locator_list: L,
    pub default_multicast_locator_list: L,
    pub available_builtin_endpoints: BuiltinEndpointSet,
    pub lease_duration: Duration,
    pub manual_liveliness_count: Count,
    pub builtin_endpoint_qos: BuiltinEndpointQos,
}
