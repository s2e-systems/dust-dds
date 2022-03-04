use crate::{
    behavior::types::Duration,
    messages::types::Count,
    structure::types::{GuidPrefix, Locator, ProtocolVersion, VendorId},
};

use crate::discovery::types::{BuiltinEndpointQos, BuiltinEndpointSet, DomainId};

pub trait RtpsSpdpDiscoveredParticipantDataAttributes {
    fn domain_id(&self) -> DomainId;
    fn domain_tag(&self) -> &str;
    fn protocol_version(&self) -> ProtocolVersion;
    fn guid_prefix(&self) -> GuidPrefix;
    fn vendor_id(&self) -> VendorId;
    fn expects_inline_qos(&self) -> bool;
    fn metatraffic_unicast_locator_list(&self) -> &[Locator];
    fn metatraffic_multicast_locator_list(&self) -> &[Locator];
    fn default_unicast_locator_list(&self) -> &[Locator];
    fn default_multicast_locator_list(&self) -> &[Locator];
    fn available_builtin_endpoints(&self) -> BuiltinEndpointSet;
    fn lease_duration(&self) -> Duration;
    fn manual_liveliness_count(&self) -> Count;
    fn builtin_endpoint_qos(&self) -> BuiltinEndpointQos;
}
