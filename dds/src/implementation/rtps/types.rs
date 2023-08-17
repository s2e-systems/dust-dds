use byteorder::ByteOrder;

use super::messages::overall_structure::{WriteBytes, WriteEndianness};
use crate::implementation::data_representation_builtin_endpoints::parameter_id_values::DEFAULT_EXPECTS_INLINE_QOS;
use std::{
    io::Read,
    ops::{Add, AddAssign, Sub, SubAssign},
};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

type Octet = u8;
type Long = i32;
type UnsignedLong = u32;

impl WriteBytes for Long {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        WriteEndianness::write_i32(buf, *self);
        4
    }
}

impl WriteBytes for UnsignedLong {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        WriteEndianness::write_u32(buf, *self);
        4
    }
}

impl WriteBytes for u16 {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        WriteEndianness::write_u16(buf, *self);
        2
    }
}

impl WriteBytes for i16 {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        WriteEndianness::write_i16(buf, *self);
        2
    }
}

/// GUID_t
/// Type used to hold globally-unique RTPS-entity identifiers. These are identifiers used to uniquely refer to each RTPS Entity in the system.
/// Must be possible to represent using 16 octets.
/// The following values are reserved by the protocol: GUID_UNKNOWN
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
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

pub const GUID_UNKNOWN: Guid = Guid::new(GUIDPREFIX_UNKNOWN, ENTITYID_UNKNOWN);

impl From<[u8; 16]> for Guid {
    fn from(value: [u8; 16]) -> Self {
        let prefix = [
            value[0], value[1], value[2], value[3], value[4], value[5], value[6], value[7],
            value[8], value[9], value[10], value[11],
        ];
        let entity_id = EntityId::new([value[12], value[13], value[14]], value[15]);
        Self { prefix, entity_id }
    }
}

impl From<Guid> for [u8; 16] {
    fn from(guid: Guid) -> Self {
        [
            guid.prefix[0],
            guid.prefix[1],
            guid.prefix[2],
            guid.prefix[3],
            guid.prefix[4],
            guid.prefix[5],
            guid.prefix[6],
            guid.prefix[7],
            guid.prefix[8],
            guid.prefix[9],
            guid.prefix[10],
            guid.prefix[11],
            guid.entity_id.entity_key[0],
            guid.entity_id.entity_key[1],
            guid.entity_id.entity_key[2],
            guid.entity_id.entity_kind,
        ]
    }
}

/// GuidPrefix_t
/// Type used to hold the prefix of the globally-unique RTPS-entity identifiers. The GUIDs of entities belonging to the same participant all have the same prefix (see 8.2.4.3).
/// Must be possible to represent using 12 octets.
/// The following values are reserved by the protocol: GUIDPREFIX_UNKNOWN
pub type GuidPrefix = [u8; 12];
pub const GUIDPREFIX_UNKNOWN: GuidPrefix = [0; 12];

impl WriteBytes for GuidPrefix {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.as_slice().read(buf).unwrap()
    }
}

/// EntityId_t
/// Type used to hold the suffix part of the globally-unique RTPS-entity identifiers. The
/// EntityId_t uniquely identifies an Entity within a Participant. Must be possible to represent using 4 octets.
/// The following values are reserved by the protocol: ENTITYID_UNKNOWN Additional pre-defined values are defined by the Discovery module in 8.5
type OctetArray3 = [Octet; 3];
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityId {
    entity_key: OctetArray3,
    entity_kind: Octet,
}

impl EntityId {
    pub const fn new(entity_key: OctetArray3, entity_kind: Octet) -> Self {
        Self {
            entity_key,
            entity_kind,
        }
    }

    pub const fn entity_key(&self) -> OctetArray3 {
        self.entity_key
    }

    pub const fn entity_kind(&self) -> Octet {
        self.entity_kind
    }
}

impl Default for EntityId {
    fn default() -> Self {
        ENTITYID_UNKNOWN
    }
}

pub const ENTITYID_UNKNOWN: EntityId = EntityId::new([0; 3], USER_DEFINED_UNKNOWN);
pub const ENTITYID_PARTICIPANT: EntityId = EntityId::new([0, 0, 0x01], BUILT_IN_PARTICIPANT);

impl WriteBytes for EntityId {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.entity_key().write_bytes(&mut buf[0..]);
        self.entity_kind().write_bytes(&mut buf[3..]);
        4
    }
}

impl WriteBytes for Octet {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[0] = *self;
        1
    }
}

// Table 9.1 - entityKind octet of an EntityId_t
pub const USER_DEFINED_UNKNOWN: Octet = 0x00;
#[allow(dead_code)]
pub const BUILT_IN_UNKNOWN: Octet = 0xc0;
pub const BUILT_IN_PARTICIPANT: Octet = 0xc1;
pub const USER_DEFINED_WRITER_WITH_KEY: Octet = 0x02;
pub const BUILT_IN_WRITER_WITH_KEY: Octet = 0xc2;
pub const USER_DEFINED_WRITER_NO_KEY: Octet = 0x03;
#[allow(dead_code)]
pub const BUILT_IN_WRITER_NO_KEY: Octet = 0xc3;
pub const USER_DEFINED_READER_WITH_KEY: Octet = 0x07;
pub const BUILT_IN_READER_WITH_KEY: Octet = 0xc7;
pub const USER_DEFINED_READER_NO_KEY: Octet = 0x04;
#[allow(dead_code)]
pub const BUILT_IN_READER_NO_KEY: Octet = 0xc4;
pub const USER_DEFINED_WRITER_GROUP: Octet = 0x08;
pub const BUILT_IN_WRITER_GROUP: Octet = 0xc8;
pub const USER_DEFINED_READER_GROUP: Octet = 0x09;
pub const BUILT_IN_READER_GROUP: Octet = 0xc9;
// Added in comparison to the RTPS standard
pub const BUILT_IN_TOPIC: Octet = 0xca;
pub const USER_DEFINED_TOPIC: Octet = 0x0a;

impl WriteBytes for OctetArray3 {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.as_slice().read(buf).unwrap()
    }
}

/// SequenceNumber_t
/// Type used to hold sequence numbers.
/// Must be possible to represent using 64 bits.
/// The following values are reserved by the protocol: SEQUENCENUMBER_UNKNOWN
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SequenceNumber {
    high: Long,
    low: UnsignedLong,
}

#[allow(dead_code)]
pub const SEQUENCENUMBER_UNKNOWN: SequenceNumber = SequenceNumber::new(-1, 0);

impl SequenceNumber {
    pub const fn new(high: Long, low: UnsignedLong) -> Self {
        Self { high, low }
    }
}
impl From<SequenceNumber> for i64 {
    fn from(value: SequenceNumber) -> Self {
        ((value.high as i64) << 32) + value.low as i64
    }
}
impl From<i64> for SequenceNumber {
    fn from(value: i64) -> Self {
        Self {
            high: (value >> 32) as Long,
            low: value as UnsignedLong,
        }
    }
}
impl Add for SequenceNumber {
    type Output = Self;
    fn add(self, rhs: SequenceNumber) -> Self::Output {
        Self::from(<i64>::from(self) + <i64>::from(rhs))
    }
}
impl Sub for SequenceNumber {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(<i64>::from(self) - <i64>::from(rhs))
    }
}
impl AddAssign<i64> for SequenceNumber {
    fn add_assign(&mut self, rhs: i64) {
        *self = Self::from(<i64>::from(*self) + rhs);
    }
}
impl Add<i64> for SequenceNumber {
    type Output = Self;
    fn add(self, rhs: i64) -> Self::Output {
        Self::from(<i64>::from(self) + rhs)
    }
}
impl SubAssign<i64> for SequenceNumber {
    fn sub_assign(&mut self, rhs: i64) {
        *self = Self::from(<i64>::from(*self) - rhs);
    }
}
impl Sub<i64> for SequenceNumber {
    type Output = Self;
    fn sub(self, rhs: i64) -> Self::Output {
        Self::from(<i64>::from(self) - rhs)
    }
}

impl WriteBytes for SequenceNumber {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.high.write_bytes(&mut buf[0..]) + self.low.write_bytes(&mut buf[4..])
    }
}

/// Locator_t
/// Type used to represent the addressing information needed to send a message to an RTPS Endpoint using one of the supported transports.
/// Should be able to hold a discriminator identifying the kind of transport, an address, and a port number. It must be possible to represent the discriminator and port number using 4 octets each, the address using 16 octets.
/// The following values are reserved by the protocol: LOCATOR_INVALID LOCATOR_KIND_INVALID LOCATOR_KIND_RESERVED LOCATOR_KIND_UDP_V4 LOCATOR_KIND_UDP_V6 LOCATOR_ADDRESS_INVALID LOCATOR_PORT_INVALID
#[derive(Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
pub struct Locator {
    kind: Long,
    port: LocatorPort,
    address: LocatorAddress,
}

impl WriteBytes for Locator {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.kind.write_bytes(&mut buf[0..]);
        self.port.write_bytes(&mut buf[4..]);
        self.address.write_bytes(&mut buf[8..]);
        24
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

impl WriteBytes for LocatorPort {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.0.write_bytes(buf)
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

impl WriteBytes for LocatorAddress {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[..self.0.len()].copy_from_slice(&self.0);
        16
    }
}

#[allow(dead_code)]
pub const LOCATOR_KIND_INVALID: Long = -1;
#[allow(dead_code)]
pub const LOCATOR_KIND_RESERVED: Long = 0;
pub const LOCATOR_KIND_UDP_V4: Long = 1;
pub const LOCATOR_KIND_UDP_V6: Long = 2;
pub const LOCATOR_PORT_INVALID: LocatorPort = LocatorPort(0);
pub const LOCATOR_ADDRESS_INVALID: LocatorAddress = LocatorAddress([0; 16]);

#[allow(dead_code)]
pub const LOCATOR_INVALID: Locator = Locator::new(
    LOCATOR_KIND_INVALID,
    LOCATOR_PORT_INVALID,
    LOCATOR_ADDRESS_INVALID,
);

impl Locator {
    pub const fn new(kind: Long, port: LocatorPort, address: LocatorAddress) -> Self {
        Self {
            kind,
            port,
            address,
        }
    }
    pub const fn kind(&self) -> Long {
        self.kind
    }
    pub const fn port(&self) -> LocatorPort {
        self.port
    }
    pub const fn address(&self) -> LocatorAddress {
        self.address
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

/// ReliabilityKind_t
/// Enumeration used to indicate the level of the reliability used for communications.
/// It can take the values: BEST_EFFORT, RELIABLE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

/// InstanceHandle_t
/// Type used to represent the identity of a data-object whose changes in value are communicated by the RTPS protocol.
// Defined elsewhere in DDS

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

impl WriteBytes for ProtocolVersion {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        buf[0] = self.major;
        buf[1] = self.minor;
        2
    }
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

impl WriteBytes for VendorId {
    fn write_bytes(&self, buf: &mut [u8]) -> usize {
        self.0.as_slice().read(buf).unwrap()
    }
}

pub const _VENDOR_ID_UNKNOWN: VendorId = VendorId([0, 0]);
pub const VENDOR_ID_S2E: VendorId = VendorId([0x01, 0x14]);

/// Additionally defined here (should move to DDS)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurabilityKind {
    Volatile,
    TransientLocal,
}

/// Additionally defined here (should move to DDS)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implementation::rtps::messages::overall_structure::into_bytes_vec;

    #[test]
    fn serialize_sequence_number() {
        let data = SequenceNumber::from(7);
        let result = into_bytes_vec(data);
        assert_eq!(
            result,
            vec![
                0, 0, 0, 0, // high (long)
                7, 0, 0, 0, // low (unsigned long)
            ]
        );
    }

    #[test]
    fn serialize_entity_id() {
        let data = EntityId::new([1, 2, 3], 0x04);
        assert_eq!(
            into_bytes_vec(data),
            vec![
            1, 2, 3, 0x04, //value (long)
        ]
        );
    }
}
