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

pub trait LocatorPIM {
    type LocatorType: LocatorType;

    const LOCATOR_INVALID: Self::LocatorType;

    const LOCATOR_KIND_INVALID: <Self::LocatorType as LocatorType>::LocatorKind;
    const LOCATOR_KIND_RESERVED: <Self::LocatorType as LocatorType>::LocatorKind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: <Self::LocatorType as LocatorType>::LocatorKind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: <Self::LocatorType as LocatorType>::LocatorKind;

    const LOCATOR_PORT_INVALID: <Self::LocatorType as LocatorType>::LocatorPort;

    const LOCATOR_ADDRESS_INVALID: <Self::LocatorType as LocatorType>::LocatorAddress;
}

pub trait LocatorType {
    type LocatorKind: Into<[u8; 4]> + From<[u8; 4]>;
    type LocatorPort: Into<[u8; 4]> + From<[u8; 4]>;
    type LocatorAddress: Into<[u8; 16]> + From<[u8; 16]>;

    fn kind(&self) -> &Self::LocatorKind;
    fn port(&self) -> &Self::LocatorPort;
    fn address(&self) -> &Self::LocatorAddress;
}

pub trait InstanceHandlePIM {
    type InstanceHandleType;
}

pub trait ProtocolVersionPIM {
    type ProtocolVersionType;
    const PROTOCOLVERSION: Self::ProtocolVersionType;
    const PROTOCOLVERSION_1_0: Self::ProtocolVersionType;
    const PROTOCOLVERSION_1_1: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_0: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_1: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_2: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_3: Self::ProtocolVersionType;
    const PROTOCOLVERSION_2_4: Self::ProtocolVersionType;
}

pub trait VendorIdPIM {
    type VendorIdType;
    const VENDOR_ID_UNKNOWN: Self::VendorIdType;
}

pub trait DataPIM {
    type DataType: AsRef<[u8]>;
}

pub trait GUIDPIM {
    type GUIDType;
    const GUID_UNKNOWN: Self::GUIDType;
}

/// Define the GUID as described in 8.2.4.1 Identifying RTPS entities: The GUID
pub trait GUIDType: From<[u8; 16]> + Into<[u8; 16]> {
    fn new(prefix: GuidPrefix, entity_id: EntityId) -> Self;
    fn prefix(&self) -> &GuidPrefix;
    fn entity_id(&self) -> &EntityId;
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
