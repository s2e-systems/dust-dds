use core::iter::FromIterator;

use crate::messages;

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///
pub trait Types {
    type GuidPrefix: Into<[u8; 12]> + From<[u8; 12]> + Copy;
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefix;

    type EntityId: Into<[u8; 4]> + From<[u8; 4]> + Copy;
    const ENTITYID_UNKNOWN: Self::EntityId;

    type SequenceNumber: Into<i64> + From<i64> + Ord + Copy;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber;

    type Locator: Locator;

    const LOCATOR_INVALID: Self::Locator;

    type TopicKind: Copy;
    const NO_KEY: Self::TopicKind;
    const WITH_KEY: Self::TopicKind;

    type ChangeKind: Copy;
    const ALIVE: Self::ChangeKind;
    const ALIVE_FILTERED: Self::ChangeKind;
    const NOT_ALIVE_DISPOSED: Self::ChangeKind;
    const NOT_ALIVE_UNREGISTERED: Self::ChangeKind;

    type ReliabilityKind: Copy + PartialEq;
    const BEST_EFFORT: Self::ReliabilityKind;
    const RELIABLE: Self::ReliabilityKind;

    type InstanceHandle: Copy;

    type ProtocolVersion: Copy;
    const PROTOCOLVERSION: Self::ProtocolVersion;
    const PROTOCOLVERSION_1_0: Self::ProtocolVersion;
    const PROTOCOLVERSION_1_1: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_0: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_1: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_2: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_3: Self::ProtocolVersion;
    const PROTOCOLVERSION_2_4: Self::ProtocolVersion;

    type VendorId: Copy;
    const VENDOR_ID_UNKNOWN: Self::VendorId;

    // Data type which is used in the RTPS CacheChange and not explicitly defined in the standard
    type Data;

    // Additions to represent lists which are used but not explicitly defined in the standard
    type SequenceNumberVector: IntoIterator<Item = Self::SequenceNumber>
        + FromIterator<Self::SequenceNumber>
        + Clone;
    type LocatorVector: IntoIterator<Item = Self::Locator>;

    type Parameter: messages::submessage_elements::Parameter<PSM = Self>;
    type ParameterVector: IntoIterator<Item = Self::Parameter>;
}

/// Define the GUID as described in 8.2.4.1 Identifying RTPS entities: The GUID
pub struct GUID<PSM: Types> {
    prefix: PSM::GuidPrefix,
    entity_id: PSM::EntityId,
}

impl<PSM: Types> Clone for GUID<PSM> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<PSM: Types> Copy for GUID<PSM> {}

impl<PSM: Types> GUID<PSM> {
    pub const GUID_UNKNOWN: Self = Self {
        prefix: PSM::GUIDPREFIX_UNKNOWN,
        entity_id: PSM::ENTITYID_UNKNOWN,
    };

    pub fn new(prefix: PSM::GuidPrefix, entity_id: PSM::EntityId) -> Self {
        Self { prefix, entity_id }
    }

    /// Get a reference to the g u i d's prefix.
    pub fn prefix(&self) -> &PSM::GuidPrefix {
        &self.prefix
    }

    /// Get a reference to the g u i d's entity id.
    pub fn entity_id(&self) -> &PSM::EntityId {
        &self.entity_id
    }
}

pub trait Locator {
    type Kind;
    type Port;
    type Address: Into<[u8; 16]> + From<[u8; 16]>;

    const LOCATOR_KIND_INVALID: Self::Kind;
    const LOCATOR_KIND_RESERVED: Self::Kind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::Kind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::Kind;
    const LOCATOR_ADDRESS_INVALID: Self::Address;
    const LOCATOR_PORT_INVALID: Self::Port;
}
