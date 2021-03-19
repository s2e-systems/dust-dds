///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///
pub trait GUID: Into<[u8; 16]> {
    const GUID_UNKNOWN: Self;
}

pub trait GuidPrefix: Into<[u8; 12]> {
    const GUID_PREFIX_UNKNOWN: Self;
}

pub trait EntityId: Into<[u8; 4]> {
    const ENTITYID_UNKNOWN: Self;
}

pub trait SequenceNumber: Into<i64> {
    const SEQUENCE_NUMBER_UNKNOWN: Self;
}

pub trait Locator {
    type Kind: Into<[u8; 4]>;
    type Port: Into<[u8; 4]>;
    type Address: Into<[u8; 16]>;

    const LOCATOR_INVALID: Self;
    const LOCATOR_KIND_INVALID: Self::Kind;
    const LOCATOR_KIND_RESERVED: Self::Kind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::Kind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::Kind;
    const LOCATOR_ADDRESS_INVALID: Self::Address;
    const LOCATOR_PORT_INVALID: Self::Address;

    fn kind(&self) -> &Self::Kind;
    fn port(&self) -> &Self::Port;
    fn address(&self) -> &Self::Address;
}

pub enum TopicKind {
    NoKey,
    WithKey,
}

pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAlive,
    Disposed,
    NotAliveUnregistered,
}

pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

pub trait InstanceHandle {}

pub trait ProtocolVersion {
    const PROTOCOL_VERSION: Self;
    const PROTOCOL_VERSION_1_0: Self;
    const PROTOCOL_VERSION_1_1: Self;
    const PROTOCOL_VERSION_2_0: Self;
    const PROTOCOL_VERSION_2_1: Self;
    const PROTOCOL_VERSION_2_2: Self;
    const PROTOCOL_VERSION_2_3: Self;
    const PROTOCOL_VERSION_2_4: Self;
}

pub trait VendorId {
    const VENDOR_ID_UNKNOWN: Self;
}
