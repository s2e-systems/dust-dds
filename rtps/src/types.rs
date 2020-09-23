/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///  

use num_derive::FromPrimitive;
use serde::{Serialize, Deserialize};

pub use rust_dds_interface::types::{InstanceHandle, Data, ReliabilityKind, TopicKind};

pub mod constants {
    use super::{VendorId, EntityId, ProtocolVersion, EntityKind, SequenceNumber, Locator, GuidPrefix, GUID};

    pub const VENDOR_ID: VendorId = [99,99];

    pub const SEQUENCE_NUMBER_UNKNOWN : SequenceNumber = std::i64::MIN;

    pub const PROTOCOL_VERSION_2_1 : ProtocolVersion = ProtocolVersion{major: 2, minor: 1};
    pub const PROTOCOL_VERSION_2_2 : ProtocolVersion = ProtocolVersion{major: 2, minor: 2};
    pub const PROTOCOL_VERSION_2_4 : ProtocolVersion = ProtocolVersion{major: 2, minor: 4};

    pub const LOCATOR_KIND_INVALID : i32 = -1;
    pub const LOCATOR_KIND_RESERVED : i32 = 0;
    #[allow(non_upper_case_globals)]
    pub const LOCATOR_KIND_UDPv4 : i32 = 1;
    #[allow(non_upper_case_globals)]
    pub const LOCATOR_KIND_UDPv6 : i32 = 2;
    pub const LOCATOR_PORT_INVALID : u32 = 0;
    pub const LOCATOR_ADDRESS_INVALID : [u8;16] = [0;16];
    pub const LOCATOR_INVALID: Locator = Locator::new(LOCATOR_KIND_INVALID, LOCATOR_PORT_INVALID, LOCATOR_ADDRESS_INVALID);

    pub const GUID_PREFIX_UNKNOWN : GuidPrefix = [0;12];

    pub const ENTITYID_UNKNOWN: EntityId = EntityId {
        entity_key: [0, 0, 0x00],
        entity_kind: EntityKind::UserDefinedUnknown,
    };

    pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: EntityKind::BuiltInParticipant,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind:EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: EntityKind::BuiltInWriterWithKey,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: EntityKind::BuiltInReaderWithKey,
    };

    pub const GUID_UNKNOWN: GUID = GUID::new(GUID_PREFIX_UNKNOWN, ENTITYID_UNKNOWN);
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GUID {
    prefix: GuidPrefix,
    entity_id: EntityId,
}

impl GUID {
    pub const fn new(prefix: GuidPrefix, entity_id: EntityId) -> GUID {
        GUID { prefix, entity_id }
    }

    pub const fn prefix(&self) -> GuidPrefix {
        self.prefix
    }

    pub const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

pub type GuidPrefix = [u8; 12];

pub type EntityKey = [u8; 3];
#[derive(FromPrimitive, Debug, Hash, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum EntityKind {
    UserDefinedUnknown = 0x00,
    UserDefinedWriterWithKey = 0x02,
    UserDefinedWriterNoKey = 0x03,
    UserDefinedReaderWithKey = 0x04,
    UserDefinedReaderNoKey = 0x07,
    UserDefinedWriterGroup = 0x08,
    UserDefinedReaderGroup = 0x09,
    BuiltInUnknown = 0xc0,
    BuiltInParticipant = 0xc1,
    BuiltInWriterWithKey = 0xc2,
    BuiltInWriterNoKey = 0xc3,
    BuiltInReaderWithKey = 0xc4,
    BuiltInReaderNoKey = 0xc7,
    BuiltInWriterGroup = 0xc8,
    BuiltInReaderGroup = 0xc9,
}

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct EntityId {
    entity_key: EntityKey,
    entity_kind: EntityKind,
}
 
impl EntityId {
    pub fn new(entity_key: EntityKey, entity_kind: EntityKind) -> EntityId {
        EntityId {
            entity_key,
            entity_kind,
        }
    }
    pub fn entity_key(&self) -> EntityKey{
        self.entity_key
    }
    pub fn entity_kind(&self) -> EntityKind{
        self.entity_kind
    }
}

pub type SequenceNumber = i64;

#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Locator {
    kind: i32,
    port: u32,
    address: [u8; 16],
}

impl Locator {
    pub const fn new(kind: i32, port: u32, address: [u8; 16]) -> Locator {
        Locator {
            kind,
            port,
            address,
        }
    }

    pub const fn new_udpv4(port: u16, address: [u8; 4]) -> Locator {
        let address: [u8;16] = [0,0,0,0,0,0,0,0,0,0,0,0,address[0],address[1],address[2],address[3]];
        Locator {
            kind: constants::LOCATOR_KIND_UDPv4,
            port: port as u32,
            address,
        }
    }

    pub fn kind(&self) -> i32 {
        self.kind
    }

    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn address(&self) -> &[u8;16] {
        &self.address
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(PartialEq, Debug, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

pub type VendorId = [u8; 2];

