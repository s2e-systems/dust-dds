use std::ops::AddAssign;

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

/// GUID_t
/// Type used to hold globally-unique RTPS-entity identifiers. These are identifiers used to uniquely refer to each RTPS Entity in the system.
/// Must be possible to represent using 16 octets.
/// The following values are reserved by the protocol: GUID_UNKNOWN
///
/// Note: Define the GUID as described in 8.2.4.1 Identifying RTPS entities: The GUID
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Guid {
    pub prefix: GuidPrefix,
    pub entity_id: EntityId,
}

#[allow(dead_code)]
pub const GUID_UNKNOWN: Guid = Guid {
    prefix: GUIDPREFIX_UNKNOWN,
    entity_id: ENTITYID_UNKNOWN,
};

impl Guid {
    pub fn new(prefix: GuidPrefix, entity_id: EntityId) -> Self {
        Self { prefix, entity_id }
    }

    pub fn prefix(&self) -> GuidPrefix {
        self.prefix
    }

    pub fn entity_id(&self) -> EntityId {
        self.entity_id
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
            guid.entity_id.entity_key[0],
            guid.entity_id.entity_key[1],
            guid.entity_id.entity_key[2],
            guid.entity_id.entity_kind,
        ]
    }
}

impl From<[u8; 16]> for Guid {
    fn from(value: [u8; 16]) -> Self {
        Guid {
            prefix: GuidPrefix([
                value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
                value[8], value[9], value[10], value[11],
            ]),
            entity_id: EntityId {
                entity_key: [value[12], value[13], value[14]],
                entity_kind: value[15],
            },
        }
    }
}

/// GuidPrefix_t
/// Type used to hold the prefix of the globally-unique RTPS-entity identifiers. The GUIDs of entities belonging to the same participant all have the same prefix (see 8.2.4.3).
/// Must be possible to represent using 12 octets.
/// The following values are reserved by the protocol: GUIDPREFIX_UNKNOWN
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct GuidPrefix(pub [u8; 12]);
pub const GUIDPREFIX_UNKNOWN: GuidPrefix = GuidPrefix([0; 12]);

impl From<GuidPrefix> for [u8; 12] {
    fn from(value: GuidPrefix) -> Self {
        value.0
    }
}

impl From<[u8; 12]> for GuidPrefix {
    fn from(value: [u8; 12]) -> Self {
        Self(value)
    }
}

/// EntityId_t
/// Type used to hold the suffix part of the globally-unique RTPS-entity identifiers. The
/// EntityId_t uniquely identifies an Entity within a Participant. Must be possible to represent using 4 octets.
/// The following values are reserved by the protocol: ENTITYID_UNKNOWN Additional pre-defined values are defined by the Discovery module in 8.5
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityId {
    pub entity_key: EntityKey,
    pub entity_kind: EntityKind,
}

impl EntityId {
    pub const fn new(entity_key: EntityKey, entity_kind: EntityKind) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    /// Get a reference to the entity id's entity key.
    pub fn entity_key(&self) -> EntityKey {
        self.entity_key
    }

    /// Get a reference to the entity id's entity kind.
    pub fn entity_kind(&self) -> EntityKind {
        self.entity_kind
    }
}

impl From<EntityId> for [u8; 4] {
    fn from(value: EntityId) -> Self {
        [
            value.entity_key[0],
            value.entity_key[1],
            value.entity_key[2],
            value.entity_kind,
        ]
    }
}

impl From<[u8; 4]> for EntityId {
    fn from(value: [u8; 4]) -> Self {
        Self {
            entity_key: [value[0], value[1], value[2]],
            entity_kind: value[3],
        }
    }
}

pub const ENTITYID_UNKNOWN: EntityId = EntityId {
    entity_key: [0; 3],
    entity_kind: USER_DEFINED_UNKNOWN,
};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
    entity_key: [0, 0, 0x01],
    entity_kind: BUILT_IN_PARTICIPANT,
};

pub type EntityKind = u8;

// Table 9.1 - entityKind octet of an EntityId_t
pub const USER_DEFINED_UNKNOWN: EntityKind = 0x00;
#[allow(dead_code)]
pub const BUILT_IN_UNKNOWN: EntityKind = 0xc0;
pub const BUILT_IN_PARTICIPANT: EntityKind = 0xc1;
pub const USER_DEFINED_WRITER_WITH_KEY: EntityKind = 0x02;
pub const BUILT_IN_WRITER_WITH_KEY: EntityKind = 0xc2;
pub const USER_DEFINED_WRITER_NO_KEY: EntityKind = 0x03;
#[allow(dead_code)]
pub const BUILT_IN_WRITER_NO_KEY: EntityKind = 0xc3;
#[allow(dead_code)]
pub const USER_DEFINED_READER_WITH_KEY: EntityKind = 0x07;
pub const BUILT_IN_READER_WITH_KEY: EntityKind = 0xc7;
#[allow(dead_code)]
pub const USER_DEFINED_READER_NO_KEY: EntityKind = 0x04;
#[allow(dead_code)]
pub const BUILT_IN_READER_NO_KEY: EntityKind = 0xc4;
pub const USER_DEFINED_WRITER_GROUP: EntityKind = 0x08;
pub const BUILT_IN_WRITER_GROUP: EntityKind = 0xc8;
pub const USER_DEFINED_READER_GROUP: EntityKind = 0x09;
pub const BUILT_IN_READER_GROUP: EntityKind = 0xc9;

pub type EntityKey = [u8; 3];

/// SequenceNumber_t
/// Type used to hold sequence numbers.
/// Must be possible to represent using 64 bits.
/// The following values are reserved by the protocol: SEQUENCENUMBER_UNKNOWN
pub type SequenceNumber = i64;
#[allow(dead_code)]
pub const SEQUENCENUMBER_UNKNOWN: SequenceNumber = i64::MIN;

/// TopicKind_t
/// Enumeration used to distinguish whether a Topic has defined some fields within to be used as the ‘key’ that identifies data-instances within the Topic. See the DDS specification for more details on keys.
/// The following values are reserved by the protocol: NO_KEY
/// WITH_KEY
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

/// ChangeKind_t
/// Enumeration used to distinguish the kind of change that was made to a data-object. Includes changes to the data or the instance state of the data-object.
/// It can take the values:
/// ALIVE, ALIVE_FILTERED, NOT_ALIVE_DISPOSED, NOT_ALIVE_UNREGISTERED
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

/// ReliabilityKind_t
/// Enumeration used to indicate the level of the reliability used for communications. It can take the values:
/// BEST_EFFORT, RELIABLE.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

/// ProtocolVersion_t
/// Type used to represent the version of the RTPS protocol. The version is composed of a major and a minor version number. See also 8.6.
/// The following values are reserved by the protocol: PROTOCOLVERSION PROTOCOLVERSION_1_0 PROTOCOLVERSION_1_1 PROTOCOLVERSION_2_0 PROTOCOLVERSION_2_1 PROTOCOLVERSION_2_2
/// PROTOCOLVERSION_2_4
/// PROTOCOLVERSION is an alias for the most recent version, in this case PROTOCOLVERSION_2_4
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
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

impl From<ProtocolVersion> for [u8; 2] {
    fn from(value: ProtocolVersion) -> Self {
        [value.major, value.minor]
    }
}
impl From<[u8; 2]> for ProtocolVersion {
    fn from(value: [u8; 2]) -> Self {
        Self {
            major: value[0],
            minor: value[1],
        }
    }
}

/// VendorId_t
/// Type used to represent the vendor of the service implementing the RTPS protocol. The possible values for the vendorId are assigned by the OMG.
/// The following values are reserved by the protocol: VENDORID_UNKNOWN
pub type VendorId = [u8; 2];
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [99, 99];

/// Count_t
/// Type used to hold a count that is incremented monotonically, used to identify message duplicates.
#[derive(Clone, Copy, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Count(pub i32);

impl AddAssign for Count {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl From<i32> for Count {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<Count> for i32 {
    fn from(value: Count) -> Self {
        value.0
    }
}
