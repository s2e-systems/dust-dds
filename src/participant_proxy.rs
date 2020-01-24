use crate::types::{ProtocolVersion, GuidPrefix, VendorId, LocatorList};
use crate::spdp::{SpdpParameter,SpdpParameterList}

pub struct ParticipantProxy {
    domain_id: u32, // TODO: Create DomainId type
    domain_tag: String,
    protocol_version: ProtocolVersion,
    guid_prefix: GuidPrefix,
    vendor_id: VendorId,
    expects_inline_qos: bool,
    //TODO: Available built-in endpoints
    //TODO: Built-in endpoints qos
    metatraffic_unicast_locator_list: LocatorList,
    metatraffic_multicast_locator_list: LocatorList,
    default_multicast_locator_list: LocatorList,
    default_unicast_locator_list: LocatorList,
    //TODO: manual liveliness count
}

// [
// ProtocolVersion(ProtocolVersion { major: 2, minor: 1 }),
// VendorId([1, 2]),
// DefaultUnicastLocator(Locator { kind: 1, port: 7411, address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1] }),
// DefaultMulticastLocator(Locator { kind: 1, port: 7401, address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1] }),
// MetatrafficUnicastLocator(Locator { kind: 1, port: 7410, address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 127, 0, 0, 1] }),
// MetatrafficMulticastLocator(Locator { kind: 1, port: 7400, address: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 239, 255, 0, 1] }),
// ParticipantGuid(GUID { prefix: [82, 191, 238, 199, 0, 0, 0, 4, 0, 0, 0, 1], entity_id: EntityId { entity_key: [0, 0, 1], entity_kind: 193 } }),
// BuiltinEndpointSet(BuiltInEndPointSet { value: 1045 })
// ParticipantLeaseDuration(Time { seconds: 11, fraction: 0 }),
// ]

impl ParticipantProxy {
    pub fn new_from_spdp_parameter_list(SpdpParameterList) -> Self {
        

    }
}