///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

pub type GuidPrefix = [u8; 12];
pub const GUIDPREFIX_UNKNOWN: GuidPrefix = [0; 12];

pub type EntityId = [u8; 4];
pub const ENTITYID_UNKNOWN: EntityId = [0; 4];
pub const ENTITYID_PARTICIPANT: EntityId = [0, 0, 0x01, 0xc1];

pub type SequenceNumber = i64;
pub const SEQUENCENUMBER_UNKNOWN: SequenceNumber = i64::MIN;

type LocatorKind = [u8; 4];
type LocatorPort = [u8; 4];
type LocatorAddress = [u8; 16];

pub const LOCATOR_KIND_INVALID: LocatorKind = [0xff, 0xff, 0xff, 0xff];
pub const LOCATOR_KIND_RESERVED: LocatorKind = [0; 4];
#[allow(non_upper_case_globals)]
pub const LOCATOR_KIND_UDPv4: LocatorKind = [0, 0, 0, 1];
#[allow(non_upper_case_globals)]
pub const LOCATOR_KIND_UDPv6: LocatorKind = [0, 0, 0, 2];
pub const LOCATOR_PORT_INVALID: LocatorPort = [0; 4];
pub const LOCATOR_ADDRESS_INVALID: LocatorAddress = [0; 16];

#[derive(Clone, Copy, PartialEq, Eq)]
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

// pub const PROTOCOLVERSION: ProtocolVersion1


pub type VendorId = [u8; 2];
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];

/// Define the GUID as described in 8.2.4.1 Identifying RTPS entities: The GUID
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GUID {
    prefix: GuidPrefix,
    entity_id: EntityId,
}

pub const GUID_UNKNOWN: GUID = GUID {
    prefix: GUIDPREFIX_UNKNOWN,
    entity_id: ENTITYID_UNKNOWN,
};

impl GUID {
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

impl From<[u8; 16]> for GUID {
    fn from(_: [u8; 16]) -> Self {
        todo!()
    }
}

impl From<GUID> for [u8; 16] {
    fn from(_: GUID) -> Self {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
