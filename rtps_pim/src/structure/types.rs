use core::iter::FromIterator;

use crate::{messages, structure};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///
pub trait Types {
    type GuidPrefix: Into<[u8; 12]> + From<[u8; 12]> + Copy + PartialEq + Send + Sync;
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefix;

    type EntityId: Into<[u8; 4]> + From<[u8; 4]> + Copy + PartialEq + Send + Sync;
    const ENTITYID_UNKNOWN: Self::EntityId;
    const ENTITYID_PARTICIPANT: Self::EntityId;

    type SequenceNumber: Into<i64> + From<i64> + Ord + Copy + Send + Sync;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber;

    type LocatorKind: PartialEq + Copy + Send + Sync;
    type LocatorPort: PartialEq + Copy + Send + Sync;
    type LocatorAddress: Into<[u8; 16]> + From<[u8; 16]> + PartialEq + Copy + Send + Sync;

    const LOCATOR_KIND_INVALID: Self::LocatorKind;
    const LOCATOR_KIND_RESERVED: Self::LocatorKind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::LocatorKind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::LocatorKind;
    const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress;
    const LOCATOR_PORT_INVALID: Self::LocatorPort;

    type InstanceHandle: Copy + Send + Sync;

    type ProtocolVersion: Copy + Send + Sync;
    const PROTOCOLVERSION: Self::ProtocolVersion;
    const PROTOCOLVERSION_1_0: Self::ProtocolVersion;
    const PROTOCOLVERSION_1_1: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_0: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_1: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_2: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_3: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_4: Self::ProtocolVersion;

    type VendorId: Copy + Send + Sync;
    const VENDOR_ID_UNKNOWN: Self::VendorId;

    // Data type which is used in the RTPS CacheChange and not explicitly defined in the standard
    type Data: Send + Sync;

    // Additions to represent lists which are used but not explicitly defined in the standard
    type SequenceNumberVector: IntoIterator<Item = Self::SequenceNumber>
        + FromIterator<Self::SequenceNumber>
        + Clone
        + Send
        + Sync;

    // Temporary solution to be able to create an independent locator vector
    type Locator: Send + Sync;
    type LocatorVector: IntoIterator<Item = Self::Locator> + Send + Sync;

    type Parameter: messages::submessage_elements::Parameter<PSM = Self> + Send + Sync;
    type ParameterVector: IntoIterator<Item = Self::Parameter> + Send + Sync;
}

/// Define the GUID as described in 8.2.4.1 Identifying RTPS entities: The GUID
pub struct GUID<PSM: structure::Types> {
    prefix: PSM::GuidPrefix,
    entity_id: PSM::EntityId,
}

impl<PSM: structure::Types> Clone for GUID<PSM> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<PSM: structure::Types> Copy for GUID<PSM> {}

impl<PSM: structure::Types> PartialEq for GUID<PSM> {
    fn eq(&self, other: &Self) -> bool {
        self.prefix == other.prefix && self.entity_id == other.entity_id
    }
}

impl<PSM: structure::Types> GUID<PSM> {
    pub const GUID_UNKNOWN: Self = Self {
        prefix: PSM::GUIDPREFIX_UNKNOWN,
        entity_id: PSM::ENTITYID_UNKNOWN,
    };

    pub fn new(prefix: PSM::GuidPrefix, entity_id: PSM::EntityId) -> Self {
        Self { prefix, entity_id }
    }

    pub fn prefix(&self) -> &PSM::GuidPrefix {
        &self.prefix
    }

    pub fn entity_id(&self) -> &PSM::EntityId {
        &self.entity_id
    }
}

pub struct Locator<PSM: Types> {
    kind: PSM::LocatorKind,
    port: PSM::LocatorPort,
    address: PSM::LocatorAddress,
}

impl<PSM: Types> Clone for Locator<PSM> {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind,
            port: self.port,
            address: self.address,
        }
    }
}

impl<PSM: Types> PartialEq for Locator<PSM> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.port == other.port && self.address == other.address
    }
}

impl<PSM: Types> Locator<PSM> {
    pub const LOCATOR_INVALID: Self = Self {
        kind: PSM::LOCATOR_KIND_INVALID,
        port: PSM::LOCATOR_PORT_INVALID,
        address: PSM::LOCATOR_ADDRESS_INVALID,
    };

    pub fn new(
        kind: PSM::LocatorKind,
        port: PSM::LocatorPort,
        address: PSM::LocatorAddress,
    ) -> Self {
        Self {
            kind,
            port,
            address,
        }
    }

    pub fn kind(&self) -> &PSM::LocatorKind {
        &self.kind
    }

    pub fn port(&self) -> &PSM::LocatorPort {
        &self.port
    }

    pub fn address(&self) -> &PSM::LocatorAddress {
        &self.address
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}
