use alloc::{string::String, vec::Vec};
use dust_dds_derive::TypeSupport;

use crate::transport::types::{EntityId, Guid, Locator, ProtocolVersion, VendorId};

/// SequenceNumber_t
/// Type used to hold sequence numbers.
/// Must be possible to represent using 64 bits.
pub type SequenceNumber = i64;
pub const SEQUENCE_NUMBER_UNKNOWN: SequenceNumber = (-1_i64) << 32;

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

/// DurabilityKind_t
/// Enumeration used to indicate the level of the durability used for communications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurabilityKind {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

pub const PROTOCOLVERSION: ProtocolVersion = PROTOCOLVERSION_2_5;
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
#[allow(dead_code)]
pub const PROTOCOLVERSION_2_4: ProtocolVersion = ProtocolVersion::new(2, 4);
pub const PROTOCOLVERSION_2_5: ProtocolVersion = ProtocolVersion::new(2, 5);

#[allow(dead_code)]
pub const VENDOR_ID_UNKNOWN: VendorId = [0, 0];
pub const VENDOR_ID_S2E: VendorId = [0x01, 0x14];

#[derive(Clone, PartialEq, Debug, Copy, PartialOrd, Eq, Ord)]
pub struct Time {
    sec: i32,
    nanosec: u32,
}

impl Time {
    /// Create a new [`Time`] with a number of seconds and nanoseconds
    pub const fn new(sec: i32, nanosec: u32) -> Self {
        let sec = sec + (nanosec / 1_000_000_000) as i32;
        let nanosec = nanosec % 1_000_000_000;
        Self { sec, nanosec }
    }

    /// Get the number of seconds contained by this time
    pub const fn sec(&self) -> i32 {
        self.sec
    }

    /// Get the number of nanoseconds contained by this time
    pub const fn nanosec(&self) -> u32 {
        self.nanosec
    }
}

/// Property type as defined in Section 9.3.2 of the DDS-RTPS specification and extended by DDS Security.
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq, Clone, Default, TypeSupport)]
#[dust_dds(extensibility = "final")]
pub struct Property {
    /// Name of the property.
    pub name: String,
    /// Value associated with that name.
    pub value: String,
    /// Indicates whether the property is intended for local use only or should be propagated by DDS discovery.
    #[dust_dds(non_serialized)]
    pub propagate: bool,
}

/// Sequence of [`Property`].
pub type PropertySeq = Vec<Property>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WriterProxy {
    pub remote_writer_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub reliability_kind: ReliabilityKind,
    pub durability_kind: DurabilityKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ReaderProxy {
    pub remote_reader_guid: Guid,
    pub remote_group_entity_id: EntityId,
    pub reliability_kind: ReliabilityKind,
    pub durability_kind: DurabilityKind,
    pub unicast_locator_list: Vec<Locator>,
    pub multicast_locator_list: Vec<Locator>,
    pub expects_inline_qos: bool,
}
