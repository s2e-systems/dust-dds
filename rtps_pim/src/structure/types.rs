///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

pub type GuidPrefix = [u8; 12];
pub const GUIDPREFIX_UNKNOWN: GuidPrefix = [0; 12];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EntityKind {
    UserDefinedUnknown,
    BuiltInUnknown,
    BuiltInParticipant,
    UserDefinedWriterWithKey,
    BuiltInWriterWithKey,
    UserDefinedWriterNoKey,
    BuiltInWriterNoKey,
    UserDefinedReaderWithKey,
    BuiltInReaderWithKey,
    UserDefinedReaderNoKey,
    BuiltInReaderNoKey,
    UserDefinedWriterGroup,
    BuiltInWriterGroup,
    UserDefinedReaderGroup,
    BuiltInReaderGroup,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct EntityId {
    entity_key: [u8; 3],
    entity_kind: EntityKind,
}

impl EntityId {
    pub const fn new(entity_key: [u8; 3], entity_kind: EntityKind) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    /// Get a reference to the entity id's entity key.
    pub fn entity_key(&self) -> &[u8; 3] {
        &self.entity_key
    }

    /// Get a reference to the entity id's entity kind.
    pub fn entity_kind(&self) -> &EntityKind {
        &self.entity_kind
    }
}

pub const ENTITYID_UNKNOWN: EntityId = EntityId {
    entity_key: [0; 3],
    entity_kind: EntityKind::UserDefinedUnknown,
};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
    entity_key: [0, 0, 0x01],
    entity_kind: EntityKind::BuiltInParticipant,
};

pub type SequenceNumber = i64;
pub const SEQUENCENUMBER_UNKNOWN: SequenceNumber = i64::MIN;

type LocatorKind = i32;
type LocatorPort = u32;
type LocatorAddress = [u8; 16];

pub const LOCATOR_KIND_INVALID: LocatorKind = -1;
pub const LOCATOR_KIND_RESERVED: LocatorKind = 0;
#[allow(non_upper_case_globals)]
pub const LOCATOR_KIND_UDPv4: LocatorKind = 1;
#[allow(non_upper_case_globals)]
pub const LOCATOR_KIND_UDPv6: LocatorKind = 2;
pub const LOCATOR_PORT_INVALID: LocatorPort = 0;
pub const LOCATOR_ADDRESS_INVALID: LocatorAddress = [0; 16];

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Locator {
    kind: LocatorKind,
    port: LocatorPort,
    address: LocatorAddress,
}

pub const LOCATOR_INVALID: Locator = Locator {
    kind: LOCATOR_KIND_INVALID,
    port: LOCATOR_PORT_INVALID,
    address: LOCATOR_ADDRESS_INVALID,
};

impl Locator {
    pub fn new(kind: LocatorKind, port: LocatorPort, address: LocatorAddress) -> Self {
        Self {
            kind,
            port,
            address,
        }
    }
    pub fn kind(&self) -> &LocatorKind {
        &self.kind
    }
    pub fn port(&self) -> &LocatorPort {
        &self.port
    }
    pub fn address(&self) -> &LocatorAddress {
        &self.address
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_4;
pub const PROTOCOLVERSION_1_0: ProtocolVersion = ProtocolVersion { major: 1, minor: 0 };
pub const PROTOCOLVERSION_1_1: ProtocolVersion = ProtocolVersion { major: 1, minor: 1 };
pub const PROTOCOLVERSION_2_0: ProtocolVersion = ProtocolVersion { major: 2, minor: 0 };
pub const PROTOCOLVERSION_2_1: ProtocolVersion = ProtocolVersion { major: 2, minor: 1 };
pub const PROTOCOLVERSION_2_2: ProtocolVersion = ProtocolVersion { major: 2, minor: 2 };
pub const PROTOCOLVERSION_2_3: ProtocolVersion = ProtocolVersion { major: 2, minor: 3 };
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion { major: 2, minor: 4 };

pub type VendorId = [u8; 2];
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [99, 99];

/// Define the GUID as described in 8.2.4.1 Identifying RTPS entities: The GUID
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Guid {
    pub prefix: GuidPrefix,
    pub entity_id: EntityId,
}

pub const GUID_UNKNOWN: Guid = Guid {
    prefix: GUIDPREFIX_UNKNOWN,
    entity_id: ENTITYID_UNKNOWN,
};

impl Guid {
    pub fn new(prefix: GuidPrefix, entity_id: EntityId) -> Self {
        Self { prefix, entity_id }
    }

    pub fn prefix(&self) -> &GuidPrefix {
        &self.prefix
    }

    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

pub type InstanceHandle = i32;
