///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///
pub trait Types {
    type Guid: Into<[u8; 16]> + From<[u8; 16]> + Copy;
    const GUID_UNKNOWN: Self::Guid;

    type GuidPrefix: Into<[u8; 12]> + From<[u8; 12]> + Copy;
    const GUIDPREFIX_UNKNOWN: Self::GuidPrefix;

    type EntityId: Into<[u8; 4]> + From<[u8; 4]> + Copy;
    const ENTITYID_UNKNOWN: Self::EntityId;

    type SequenceNumber: Into<i64> + From<i64> + Copy + Ord;
    const SEQUENCE_NUMBER_UNKNOWN: Self::SequenceNumber;

    type LocatorKind;
    type LocatorPort;
    type LocatorAddress: Into<[u8; 16]>;
    type Locator;
    const LOCATOR_INVALID: Self::Locator;
    const LOCATOR_KIND_INVALID: Self::LocatorKind;
    const LOCATOR_KIND_RESERVED: Self::LocatorKind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::LocatorKind;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::LocatorKind;
    const LOCATOR_ADDRESS_INVALID: Self::LocatorAddress;
    const LOCATOR_PORT_INVALID: Self::LocatorPort;

    type TopicKind: Copy;
    const NO_KEY: Self::TopicKind;
    const WITH_KEY: Self::TopicKind;

    type ChangeKind: Copy;
    const ALIVE: Self::ChangeKind;
    const ALIVE_FILTERED: Self::ChangeKind;
    const NOT_ALIVE_DISPOSED: Self::ChangeKind;
    const NOT_ALIVE_UNREGISTERED: Self::ChangeKind;

    type ReliabilityKind: Copy;
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
}
