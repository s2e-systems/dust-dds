use super::{
    error::RtpsResult,
    messages::overall_structure::{Endianness, TryReadFromBytes, WriteIntoBytes},
};
use crate::serialized_payload::cdr::{deserialize::CdrDeserialize, serialize::CdrSerialize};
use network_interface::Addr;
use std::{
    io::{Read, Write},
    net::IpAddr,
};

///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.5
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///

type Octet = u8;
pub type Long = i32;
pub type UnsignedLong = u32;

impl WriteIntoBytes for Octet {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(&[*self]).expect("buffer big enough");
    }
}

impl WriteIntoBytes for Long {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl WriteIntoBytes for UnsignedLong {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl WriteIntoBytes for u16 {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl WriteIntoBytes for i16 {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self.to_le_bytes().as_slice())
            .expect("buffer big enough");
    }
}

impl<const N: usize> WriteIntoBytes for [Octet; N] {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self).expect("buffer big enough");
    }
}

impl WriteIntoBytes for &[u8] {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        buf.write_all(self).expect("buffer big enough");
    }
}

/// GUID_t
/// Type used to hold globally-unique RTPS-entity identifiers. These are identifiers used to uniquely refer to each RTPS Entity in the system.
/// Must be possible to represent using 16 octets.
/// The following values are reserved by the protocol: GUID_UNKNOWN
#[derive(Clone, Copy, PartialEq, Eq, Debug, CdrSerialize, CdrDeserialize)]
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

impl TryReadFromBytes for GuidPrefix {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut guid_prefix = [0; 12];
        data.read_exact(&mut guid_prefix)?;
        Ok(guid_prefix)
    }
}

/// EntityId_t
/// Type used to hold the suffix part of the globally-unique RTPS-entity identifiers. The
/// EntityId_t uniquely identifies an Entity within a Participant. Must be possible to represent using 4 octets.
/// The following values are reserved by the protocol: ENTITYID_UNKNOWN Additional pre-defined values are defined by the Discovery module in 8.5
type OctetArray3 = [Octet; 3];
#[derive(Clone, Copy, PartialEq, Eq, Debug, CdrSerialize, CdrDeserialize)]
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

    pub fn from_bytes(data: &[u8]) -> Self {
        Self::new([data[0], data[1], data[2]], data[3])
    }
}

impl TryReadFromBytes for EntityId {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut entity_key = [0; 3];
        let mut entity_kind = [0; 1];
        data.read_exact(&mut entity_key)?;
        data.read_exact(&mut entity_kind)?;
        Ok(Self {
            entity_key,
            entity_kind: entity_kind[0],
        })
    }
}

impl Default for EntityId {
    fn default() -> Self {
        ENTITYID_UNKNOWN
    }
}

pub const ENTITYID_UNKNOWN: EntityId = EntityId::new([0; 3], USER_DEFINED_UNKNOWN);
pub const ENTITYID_PARTICIPANT: EntityId = EntityId::new([0, 0, 0x01], BUILT_IN_PARTICIPANT);

impl WriteIntoBytes for EntityId {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self.entity_key().write_into_bytes(buf);
        self.entity_kind().write_into_bytes(buf);
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

/// SequenceNumber_t
/// Type used to hold sequence numbers.
/// Must be possible to represent using 64 bits.
pub type SequenceNumber = i64;

impl TryReadFromBytes for SequenceNumber {
    fn try_read_from_bytes(data: &mut &[u8], endianness: &Endianness) -> RtpsResult<Self> {
        let high = i32::try_read_from_bytes(data, endianness)?;
        let low = u32::try_read_from_bytes(data, endianness)?;
        let value = ((high as i64) << 32) + low as i64;
        Ok(value)
    }
}

impl WriteIntoBytes for SequenceNumber {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        let high = (*self >> 32) as Long;
        let low = *self as UnsignedLong;
        high.write_into_bytes(buf);
        low.write_into_bytes(buf);
    }
}

/// Locator_t
/// Type used to represent the addressing information needed to send a message to an RTPS Endpoint using one of the supported transports.
/// Should be able to hold a discriminator identifying the kind of transport, an address, and a port number. It must be possible to represent the discriminator and port number using 4 octets each, the address using 16 octets.
/// The following values are reserved by the protocol: LOCATOR_INVALID LOCATOR_KIND_INVALID LOCATOR_KIND_RESERVED LOCATOR_KIND_UDP_V4 LOCATOR_KIND_UDP_V6 LOCATOR_ADDRESS_INVALID LOCATOR_PORT_INVALID
#[derive(Clone, Copy, PartialEq, Eq, Debug, CdrSerialize, CdrDeserialize)]
pub struct Locator {
    kind: Long,
    port: UnsignedLong,
    address: [Octet; 16],
}

impl WriteIntoBytes for Locator {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self.kind.write_into_bytes(buf);
        self.port.write_into_bytes(buf);
        self.address.write_into_bytes(buf);
    }
}

impl TryReadFromBytes for Locator {
    fn try_read_from_bytes(data: &mut &[u8], endianness: &Endianness) -> RtpsResult<Self> {
        let kind = i32::try_read_from_bytes(data, endianness)?;
        let port = u32::try_read_from_bytes(data, endianness)?;
        let mut address = [0; 16];
        data.read_exact(&mut address)?;
        Ok(Self::new(kind, port, address))
    }
}

#[allow(dead_code)]
pub const LOCATOR_KIND_INVALID: Long = -1;
#[allow(dead_code)]
pub const LOCATOR_KIND_RESERVED: Long = 0;
pub const LOCATOR_KIND_UDP_V4: Long = 1;
pub const LOCATOR_KIND_UDP_V6: Long = 2;
pub const LOCATOR_PORT_INVALID: UnsignedLong = 0;
pub const LOCATOR_ADDRESS_INVALID: [Octet; 16] = [0; 16];

#[allow(dead_code)]
pub const LOCATOR_INVALID: Locator = Locator::new(
    LOCATOR_KIND_INVALID,
    LOCATOR_PORT_INVALID,
    LOCATOR_ADDRESS_INVALID,
);

impl Locator {
    pub const fn new(kind: Long, port: UnsignedLong, address: [Octet; 16]) -> Self {
        Self {
            kind,
            port,
            address,
        }
    }
    pub const fn kind(&self) -> Long {
        self.kind
    }
    pub const fn port(&self) -> UnsignedLong {
        self.port
    }
    pub const fn address(&self) -> [Octet; 16] {
        self.address
    }

    pub fn from_ip_and_port(ip_addr: &Addr, port: u32) -> Self {
        match ip_addr.ip() {
            IpAddr::V4(a) => Self {
                kind: LOCATOR_KIND_UDP_V4,
                port,
                address: [
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    a.octets()[0],
                    a.octets()[1],
                    a.octets()[2],
                    a.octets()[3],
                ],
            },
            IpAddr::V6(a) => Self {
                kind: LOCATOR_KIND_UDP_V6,
                port,
                address: a.octets(),
            },
        }
    }
}

/// TopicKind_t
/// Enumeration used to distinguish whether a Topic has defined some fields within to be used as the 'key' that identifies data-instances within the Topic. See the DDS specification for more details on keys.
/// The following values are reserved by the protocol: NO_KEY, WITH_KEY
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

/// ChangeCount_t
/// Type used to hold a counter representing the number of HistoryCache changes that belong to a certain category.
/// For example, the number of changes that have been filtered for an RTPS Reader endpoint.
#[allow(dead_code)]
pub struct ChangeCount {
    high: Long,
    low: UnsignedLong,
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, CdrSerialize, CdrDeserialize)]
pub struct ProtocolVersion {
    bytes: [u8; 2],
}

impl TryReadFromBytes for ProtocolVersion {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut bytes = [0; 2];
        data.read_exact(&mut bytes)?;
        Ok(Self { bytes })
    }
}

impl WriteIntoBytes for ProtocolVersion {
    fn write_into_bytes(&self, buf: &mut dyn Write) {
        self.bytes.write_into_bytes(buf);
    }
}

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_4;
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_0: ProtocolVersion = ProtocolVersion::new(1, 0);
#[allow(dead_code)]
pub const PROTOCOLVERSION_1_1: ProtocolVersion = ProtocolVersion::new(1, 1);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_0: ProtocolVersion = ProtocolVersion::new(2, 0);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_1: ProtocolVersion = ProtocolVersion::new(2, 1);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_2: ProtocolVersion = ProtocolVersion::new(2, 2);
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_3: ProtocolVersion = ProtocolVersion::new(2, 3);
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion::new(2, 4);

impl ProtocolVersion {
    pub const fn new(major: Octet, minor: Octet) -> Self {
        Self {
            bytes: [major, minor],
        }
    }
    pub const fn _major(&self) -> Octet {
        self.bytes[0]
    }
    pub const fn _minor(&self) -> Octet {
        self.bytes[1]
    }
}

/// VendorId_t
/// Type used to represent the vendor of the service implementing the RTPS protocol. The possible values for the vendorId are assigned by the OMG.
/// The following values are reserved by the protocol: VENDORID_UNKNOWN
pub type VendorId = [Octet; 2];

impl TryReadFromBytes for VendorId {
    fn try_read_from_bytes(data: &mut &[u8], _endianness: &Endianness) -> RtpsResult<Self> {
        let mut bytes = [0; 2];
        data.read_exact(&mut bytes)?;
        Ok(bytes)
    }
}

#[allow(dead_code)]
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [0x01, 0x14];

#[cfg(test)]
mod tests {
    use crate::rtps::messages::overall_structure::write_into_bytes_vec;

    use super::*;

    #[test]
    fn deserialize_u16() {
        let mut data = &[7, 0, 123][..];
        let result = u16::try_read_from_bytes(&mut data, &Endianness::LittleEndian).unwrap();
        assert_eq!(result, 7);
        assert_eq!(data, &[123]);
    }

    #[test]
    fn serialize_sequence_number() {
        let data: SequenceNumber = 7;
        let result = write_into_bytes_vec(data);
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
            write_into_bytes_vec(data),
            vec![
            1, 2, 3, 0x04, //value (long)
        ]
        );
    }
}
