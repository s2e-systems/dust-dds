use crate::{
    discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId},
    messages::types::Count,
    structure::types::{GuidPrefix, ProtocolVersion, VendorId},
};

pub trait SPDPdiscoveredParticipantData {
    type LocatorListType;

    fn domain_id(&self) -> DomainId;
    fn domain_tag(&self) -> &str;
    fn protocol_version(&self) -> ProtocolVersion;
    fn guid_prefix(&self) -> GuidPrefix;
    fn vendor_id(&self) -> VendorId;
    fn expects_inline_qos(&self) -> bool;
    fn metatraffic_unicast_locator_list(&self) -> Self::LocatorListType;
    fn metatraffic_multicast_locator_list(&self) -> Self::LocatorListType;
    fn default_unicast_locator_list(&self) -> Self::LocatorListType;
    fn default_multicast_locator_list(&self) -> Self::LocatorListType;
    fn available_builtin_endpoints(&self) -> BuiltinEndpointSet;
    fn manual_liveliness_count(&self) -> Count;
    fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos;
}
