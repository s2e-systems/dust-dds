use std::ops::{Add, AddAssign, Sub, SubAssign};

use crate::implementation::{
    data_representation_builtin_endpoints::parameter_id_values::DEFAULT_EXPECTS_INLINE_QOS,
    rtps_udp_psm::mapping_traits::NumberOfBytes,
};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

/// GUID_t
/// Type used to hold globally-unique RTPS-entity identifiers. These are identifiers used to uniquely refer to each RTPS Entity in the system.
/// Must be possible to represent using 16 octets.
/// The following values are reserved by the protocol: GUID_UNKNOWN
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize)]
pub struct Guid {
    prefix: GuidPrefix,
    entity_id: EntityId,
}

impl Guid {
    pub const fn new(prefix: GuidPrefix, entity_id: EntityId) -> Self {
        Self { prefix, entity_id }
    }

    pub const fn prefix(&self) -> GuidPrefix {
        self.prefix
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

pub const GUID_UNKNOWN: Guid = Guid {
    prefix: GUIDPREFIX_UNKNOWN,
    entity_id: ENTITYID_UNKNOWN,
};

impl From<[u8; 16]> for Guid {
    fn from(value: [u8; 16]) -> Self {
        let prefix = GuidPrefix::new([
            value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
            value[8], value[9], value[10], value[11],
        ]);
        let entity_id = EntityId::new(
            EntityKey::new([value[12], value[13], value[14]]),
            EntityKind(value[15]),
        );
        Self { prefix, entity_id }
    }
}

impl From<Guid> for [u8; 16] {
    fn from(guid: Guid) -> Self {
        [
            guid.prefix.0[0],
            guid.prefix.0[1],
            guid.prefix.0[2],
            guid.prefix.0[3],
            guid.prefix.0[4],
            guid.prefix.0[5],
            guid.prefix.0[6],
            guid.prefix.0[7],
            guid.prefix.0[8],
            guid.prefix.0[9],
            guid.prefix.0[10],
            guid.prefix.0[11],
            guid.entity_id.entity_key.0[0],
            guid.entity_id.entity_key.0[1],
            guid.entity_id.entity_key.0[2],
            guid.entity_id.entity_kind.0,
        ]
    }
}

/// GuidPrefix_t
/// Type used to hold the prefix of the globally-unique RTPS-entity identifiers. The GUIDs of entities belonging to the same participant all have the same prefix (see 8.2.4.3).
/// Must be possible to represent using 12 octets.
/// The following values are reserved by the protocol: GUIDPREFIX_UNKNOWN
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, derive_more::From, derive_more::Into, serde::Serialize)]
pub struct GuidPrefix([u8; 12]);
pub const GUIDPREFIX_UNKNOWN: GuidPrefix = GuidPrefix([0; 12]);

impl GuidPrefix {
    pub const fn new(value: [u8; 12]) -> Self {
        Self(value)
    }
}

/// EntityId_t
/// Type used to hold the suffix part of the globally-unique RTPS-entity identifiers. The
/// EntityId_t uniquely identifies an Entity within a Participant. Must be possible to represent using 4 octets.
/// The following values are reserved by the protocol: ENTITYID_UNKNOWN Additional pre-defined values are defined by the Discovery module in 8.5
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityId {
    entity_key: EntityKey,
    entity_kind: EntityKind,
}

impl EntityId {
    pub const fn new(entity_key: EntityKey, entity_kind: EntityKind) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    pub const fn entity_key(&self) -> EntityKey {
        self.entity_key
    }

    pub const fn entity_kind(&self) -> EntityKind {
        self.entity_kind
    }
}

impl From<EntityId> for [u8; 4] {
    fn from(value: EntityId) -> Self {
        [
            value.entity_key.0[0],
            value.entity_key.0[1],
            value.entity_key.0[2],
            value.entity_kind.0,
        ]
    }
}

impl Default for EntityId {
    fn default() -> Self {
        ENTITYID_UNKNOWN
    }
}

pub const ENTITYID_UNKNOWN: EntityId = EntityId {
    entity_key: EntityKey::new([0; 3]),
    entity_kind: USER_DEFINED_UNKNOWN,
};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
    entity_key: EntityKey::new([0, 0, 0x01]),
    entity_kind: BUILT_IN_PARTICIPANT,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct EntityKind(u8);

impl EntityKind {
    pub const fn new(value: u8) -> Self {
        Self(value)
    }
}

impl From<EntityKind> for u8 {
    fn from(value: EntityKind) -> Self {
        value.0
    }
}

// Table 9.1 - entityKind octet of an EntityId_t
pub const USER_DEFINED_UNKNOWN: EntityKind = EntityKind(0x00);
#[allow(dead_code)]
pub const BUILT_IN_UNKNOWN: EntityKind = EntityKind(0xc0);
pub const BUILT_IN_PARTICIPANT: EntityKind = EntityKind(0xc1);
pub const USER_DEFINED_WRITER_WITH_KEY: EntityKind = EntityKind(0x02);
pub const BUILT_IN_WRITER_WITH_KEY: EntityKind = EntityKind(0xc2);
pub const USER_DEFINED_WRITER_NO_KEY: EntityKind = EntityKind(0x03);
#[allow(dead_code)]
pub const BUILT_IN_WRITER_NO_KEY: EntityKind = EntityKind(0xc3);
pub const USER_DEFINED_READER_WITH_KEY: EntityKind = EntityKind(0x07);
pub const BUILT_IN_READER_WITH_KEY: EntityKind = EntityKind(0xc7);
pub const USER_DEFINED_READER_NO_KEY: EntityKind = EntityKind(0x04);
#[allow(dead_code)]
pub const BUILT_IN_READER_NO_KEY: EntityKind = EntityKind(0xc4);
pub const USER_DEFINED_WRITER_GROUP: EntityKind = EntityKind(0x08);
pub const BUILT_IN_WRITER_GROUP: EntityKind = EntityKind(0xc8);
pub const USER_DEFINED_READER_GROUP: EntityKind = EntityKind(0x09);
pub const BUILT_IN_READER_GROUP: EntityKind = EntityKind(0xc9);
// Added in comparison to the RTPS standard
pub const BUILT_IN_TOPIC: EntityKind = EntityKind(0xca);
pub const USER_DEFINED_TOPIC: EntityKind = EntityKind(0x0a);

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Debug,
    derive_more::Into,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct EntityKey([u8; 3]);

impl EntityKey {
    pub const fn new(value: [u8; 3]) -> Self {
        Self(value)
    }
}

/// SequenceNumber_t
/// Type used to hold sequence numbers.
/// Must be possible to represent using 64 bits.
/// The following values are reserved by the protocol: SEQUENCENUMBER_UNKNOWN
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Debug,
    derive_more::Into,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
)]
pub struct SequenceNumber(i64);

impl SequenceNumber {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }
}
impl NumberOfBytes for SequenceNumber {
    fn number_of_bytes(&self) -> usize {
        8
    }
}

#[allow(dead_code)]
pub const SEQUENCENUMBER_UNKNOWN: SequenceNumber = SequenceNumber(i64::MIN);

impl AddAssign<i64> for SequenceNumber {
    fn add_assign(&mut self, rhs: i64) {
        self.0 += rhs;
    }
}
impl Add<i64> for SequenceNumber {
    type Output = SequenceNumber;

    fn add(self, rhs: i64) -> Self::Output {
        SequenceNumber(self.0 + rhs)
    }
}
impl SubAssign<i64> for SequenceNumber {
    fn sub_assign(&mut self, rhs: i64) {
        self.0 -= rhs
    }
}
impl Sub<i64> for SequenceNumber {
    type Output = SequenceNumber;

    fn sub(self, rhs: i64) -> Self::Output {
        SequenceNumber(self.0 - rhs)
    }
}

/// TopicKind_t
/// Enumeration used to distinguish whether a Topic has defined some fields within to be used as the ‘key’ that identifies data-instances within the Topic. See the DDS specification for more details on keys.
/// The following values are reserved by the protocol: NO_KEY
/// WITH_KEY
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

/// ChangeKind_t
/// Enumeration used to distinguish the kind of change that was made to a data-object. Includes changes to the data or the instance state of the data-object.
/// It can take the values:
/// ALIVE, ALIVE_FILTERED, NOT_ALIVE_DISPOSED, NOT_ALIVE_UNREGISTERED
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[allow(dead_code)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
    NotAliveDisposedUnregistered,
}

/// ProtocolVersion_t
/// Type used to represent the version of the RTPS protocol. The version is composed of a major and a minor version number. See also 8.6.
/// The following values are reserved by the protocol: PROTOCOLVERSION PROTOCOLVERSION_1_0 PROTOCOLVERSION_1_1 PROTOCOLVERSION_2_0 PROTOCOLVERSION_2_1 PROTOCOLVERSION_2_2
/// PROTOCOLVERSION_2_4
/// PROTOCOLVERSION is an alias for the most recent version, in this case PROTOCOLVERSION_2_4
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProtocolVersion {
    major: u8,
    minor: u8,
}

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_4;
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_0: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_1: ProtocolVersion = ProtocolVersion { major: 1, minor: 1 };
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_0: ProtocolVersion = ProtocolVersion { major: 2, minor: 0 };
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_1: ProtocolVersion = ProtocolVersion { major: 2, minor: 1 };
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_2: ProtocolVersion = ProtocolVersion { major: 2, minor: 2 };
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_3: ProtocolVersion = ProtocolVersion { major: 2, minor: 3 };
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion { major: 2, minor: 4 };

impl ProtocolVersion {
    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
    pub const fn major(&self) -> u8 {
        self.major
    }
    pub const fn minor(&self) -> u8 {
        self.minor
    }
}

/// VendorId_t
/// Type used to represent the vendor of the service implementing the RTPS protocol. The possible values for the vendorId are assigned by the OMG.
/// The following values are reserved by the protocol: VENDORID_UNKNOWN
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, derive_more::Into,
)]
pub struct VendorId([u8; 2]);

impl VendorId {
    pub const fn new(value: [u8; 2]) -> Self {
        Self(value)
    }
}

pub const VENDOR_ID_UNKNOWN: VendorId = VendorId([0, 0]);
pub const VENDOR_ID_S2E: VendorId = VendorId([0x01, 0x14]);

/// Count_t
/// Type used to hold a count that is incremented monotonically, used to identify message duplicates.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    serde::Serialize,
    serde::Deserialize,
    Default,
    derive_more::Into,
    derive_more::Add,
    derive_more::AddAssign,
    derive_more::Sub,
    derive_more::SubAssign,
)]
pub struct Count(i32);

impl Count {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }
    pub const fn wrapping_add(self, rhs: i32) -> Self {
        Self(self.0.wrapping_add(rhs))
    }
}
impl PartialOrd<Count> for Count {
    fn partial_cmp(&self, other: &Count) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

/// Locator_t
/// Type used to represent the addressing information needed to send a message to an RTPS Endpoint using one of the supported transports.
/// Should be able to hold a discriminator identifying the kind of transport, an address, and a port number. It must be possible to represent the discriminator and port number using 4 octets each, the address using 16 octets.
/// The following values are reserved by the protocol: LOCATOR_INVALID LOCATOR_KIND_INVALID LOCATOR_KIND_RESERVED LOCATOR_KIND_UDP_V4 LOCATOR_KIND_UDP_V6 LOCATOR_ADDRESS_INVALID LOCATOR_PORT_INVALID
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Locator {
    kind: LocatorKind,
    port: LocatorPort,
    address: LocatorAddress,
}

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, derive_more::Into,
)]
pub struct LocatorKind(i32);

impl LocatorKind {
    pub const fn new(value: i32) -> Self {
        Self(value)
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, derive_more::Into,
)]
pub struct LocatorPort(u32);

impl LocatorPort {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, derive_more::Into,
)]
pub struct LocatorAddress([u8; 16]);

impl LocatorAddress {
    pub const fn new(value: [u8; 16]) -> Self {
        Self(value)
    }
}

#[allow(dead_code)]
pub const LOCATOR_KIND_INVALID: LocatorKind = LocatorKind(-1);
#[allow(dead_code)]
pub const LOCATOR_KIND_RESERVED: LocatorKind = LocatorKind(0);
pub const LOCATOR_KIND_UDP_V4: LocatorKind = LocatorKind(1);
pub const LOCATOR_KIND_UDP_V6: LocatorKind = LocatorKind(2);
pub const LOCATOR_PORT_INVALID: LocatorPort = LocatorPort(0);
pub const LOCATOR_ADDRESS_INVALID: LocatorAddress = LocatorAddress([0; 16]);

#[allow(dead_code)]
pub const LOCATOR_INVALID: Locator = Locator {
    kind: LOCATOR_KIND_INVALID,
    port: LOCATOR_PORT_INVALID,
    address: LOCATOR_ADDRESS_INVALID,
};

impl Locator {
    pub const fn new(kind: LocatorKind, port: LocatorPort, address: LocatorAddress) -> Self {
        Self {
            kind,
            port,
            address,
        }
    }
    pub const fn kind(&self) -> LocatorKind {
        self.kind
    }
    pub const fn port(&self) -> LocatorPort {
        self.port
    }
    pub const fn address(&self) -> LocatorAddress {
        self.address
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurabilityKind {
    Volatile,
    TransientLocal,
}

#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    serde::Deserialize,
    serde::Serialize,
    derive_more::Into,
    derive_more::From,
)]
pub struct ExpectsInlineQos(bool);
impl Default for ExpectsInlineQos {
    fn default() -> Self {
        Self(DEFAULT_EXPECTS_INLINE_QOS)
    }
}
