use crate::{
    implementation::rtps::types::{EntityId, ENTITYID_UNKNOWN},
    infrastructure::time::Duration,
};

// Constant value from Table 9.13 - ParameterId Values
pub const _PID_PAD: u16 = 0x0000;
pub const _PID_SENTINEL: u16 = 0x0001;
pub const PID_USER_DATA: u16 = 0x002c;
pub const PID_TOPIC_NAME: u16 = 0x0005;
pub const PID_TYPE_NAME: u16 = 0x0007;
pub const PID_GROUP_DATA: u16 = 0x002d;
pub const PID_TOPIC_DATA: u16 = 0x002e;
pub const PID_DURABILITY: u16 = 0x001d;
pub const _PID_DURABILITY_SERVICE: u16 = 0x001e;
pub const PID_DEADLINE: u16 = 0x0023;
pub const PID_LATENCY_BUDGET: u16 = 0x0027;
pub const PID_LIVELINESS: u16 = 0x001b;
pub const PID_RELIABILITY: u16 = 0x001a;
pub const PID_LIFESPAN: u16 = 0x002b;
pub const PID_DESTINATION_ORDER: u16 = 0x0025;
pub const PID_HISTORY: u16 = 0x0040;
pub const PID_RESOURCE_LIMITS: u16 = 0x0041;
pub const PID_OWNERSHIP: u16 = 0x001f;
pub const _PID_OWNERSHIP_STRENGTH: u16 = 0x0006;
pub const PID_PRESENTATION: u16 = 0x0021;
pub const PID_PARTITION: u16 = 0x0029;
pub const PID_TIME_BASED_FILTER: u16 = 0x0004;
pub const PID_TRANSPORT_PRIORITY: u16 = 0x0049;
pub const PID_DOMAIN_ID: u16 = 0x000f;
pub const PID_DOMAIN_TAG: u16 = 0x4014;
pub const PID_PROTOCOL_VERSION: u16 = 0x0015;
pub const PID_VENDORID: u16 = 0x0016;
pub const PID_UNICAST_LOCATOR: u16 = 0x002f;
pub const PID_MULTICAST_LOCATOR: u16 = 0x0030;
pub const PID_DEFAULT_UNICAST_LOCATOR: u16 = 0x0031;
pub const PID_DEFAULT_MULTICAST_LOCATOR: u16 = 0x0048;
pub const PID_METATRAFFIC_UNICAST_LOCATOR: u16 = 0x0032;
pub const PID_METATRAFFIC_MULTICAST_LOCATOR: u16 = 0x0033;
pub const PID_EXPECTS_INLINE_QOS: u16 = 0x0043;
pub const PID_PARTICIPANT_MANUAL_LIVELINESS_COUNT: u16 = 0x0034;
pub const PID_PARTICIPANT_LEASE_DURATION: u16 = 0x0002;
pub const _PID_CONTENT_FILTER_PROPERTY: u16 = 0x0035;
pub const PID_PARTICIPANT_GUID: u16 = 0x0050;
pub const _PID_GROUP_GUID: u16 = 0x0052;
pub const PID_BUILTIN_ENDPOINT_SET: u16 = 0x0058;
pub const PID_BUILTIN_ENDPOINT_QOS: u16 = 0x0077;
pub const _PID_PROPERTY_LIST: u16 = 0x0059;
pub const PID_TYPE_MAX_SIZE_SERIALIZED: u16 = 0x0060;
pub const _PID_ENTITY_NAME: u16 = 0x0062;
pub const PID_ENDPOINT_GUID: u16 = 0x005a;
// Following PID is not defined in standard
// (but its listed in "Table 9.14 - ParameterId mapping and default values")
pub const PID_DATA_MAX_SIZE_SERIALIZED: u16 = PID_TYPE_MAX_SIZE_SERIALIZED;
// Following PID is listed in "Table 9.19 – Deprecated ParameterId Values" but
// also in "Table 9.14 - ParameterId mapping and default values"
pub const PID_GROUP_ENTITYID: u16 = 0x0053;

// Constant value from Table 9.14 - ParameterId mapping and default values
// that are not N/A and not See DDS specification
// PID_DOMAIN_ID is omitted since the default is dynamic
pub const DEFAULT_DOMAIN_TAG: &str = "";
pub const DEFAULT_EXPECTS_INLINE_QOS: bool = false;
pub const DEFAULT_PARTICIPANT_LEASE_DURATION: Duration = Duration::new(100, 0);
pub const _DEFAULT_GROUP_ENTITYID: EntityId = ENTITYID_UNKNOWN;
