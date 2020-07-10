/// 
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///  

use std::convert::TryFrom;
use crate::primitive_types::{Long, ULong, };
use crate::inline_qos_types::StatusInfo;


pub mod constants {
    use super::{VendorId, EntityId, ProtocolVersion, EntityKind, SequenceNumber};

    pub const VENDOR_ID: VendorId = [99,99];

    pub const SEQUENCE_NUMBER_UNKNOWN : SequenceNumber = std::i64::MIN;

    pub const PROTOCOL_VERSION_2_1 : ProtocolVersion = ProtocolVersion{major: 2, minor: 1};
    pub const PROTOCOL_VERSION_2_2 : ProtocolVersion = ProtocolVersion{major: 2, minor: 2};
    pub const PROTOCOL_VERSION_2_4 : ProtocolVersion = ProtocolVersion{major: 2, minor: 4};

    pub const USER_DEFINED_UNKNOWN: EntityKind = 0x00;
    pub const USER_DEFINED_WRITER_WITH_KEY: EntityKind = 0x02;
    pub const USER_DEFINED_WRITER_NO_KEY: EntityKind = 0x03;
    pub const USER_DEFINED_READER_WITH_KEY: EntityKind = 0x04;
    pub const USER_DEFINED_READER_NO_KEY: EntityKind = 0x07;
    pub const USER_DEFINED_WRITER_GROUP: EntityKind = 0x08;
    pub const USER_DEFINED_READER_GROUP: EntityKind = 0x09;
    pub const BUILTIN_UNKNOWN: EntityKind = 0xc0;
    pub const BUILTIN_PARTICIPANT: EntityKind = 0xc1;
    pub const BUILTIN_WRITER_WITH_KEY: EntityKind = 0xc2;
    pub const BUILTIN_WRITER_NO_KEY: EntityKind = 0xc3;
    pub const BUILTIN_READER_WITH_KEY: EntityKind = 0xc4;
    pub const BUILTIN_READER_NO_KEY: EntityKind = 0xc7;
    pub const BUILTIN_WRITER_GROUP: EntityKind = 0xc8;
    pub const BUILTIN_READER_GROUP: EntityKind = 0xc9;

    pub const ENTITYID_UNKNOWN: EntityId = EntityId {
        entity_key: [0, 0, 0x00],
        entity_kind: USER_DEFINED_UNKNOWN,
    };

    pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: BUILTIN_PARTICIPANT,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x02],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x03],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0, 0x04],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
        entity_key: [0, 0x01, 0x00],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: BUILTIN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
        entity_key: [0, 0x02, 0x00],
        entity_kind: BUILTIN_READER_WITH_KEY,
    };
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub struct GUID {
    prefix: GuidPrefix,
    entity_id: EntityId,
}

impl GUID {
    pub fn new(prefix: GuidPrefix, entity_id: EntityId) -> GUID {
        GUID { prefix, entity_id }
    }

    pub fn prefix(&self) -> &GuidPrefix {
        &self.prefix
    }

    pub fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }
}

pub type GuidPrefix = [u8; 12];

pub type EntityKey = [u8; 3];
pub type EntityKind = u8;
#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
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

#[derive(PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Locator {
    pub kind: Long,
    pub port: ULong,
    pub address: [u8; 16],
}

impl Locator {
    pub fn new(kind: i32, port: u32, address: [u8; 16]) -> Locator {
        Locator {
            kind,
            port,
            address,
        }
    }
}

pub enum TopicKind {
    NoKey,
    WithKey,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

impl TryFrom<StatusInfo> for ChangeKind {
    type Error = &'static str;

    fn try_from(status_info: StatusInfo) -> Result<Self, Self::Error> {
        if status_info.disposed_flag() && !status_info.unregistered_flag() && !status_info.filtered_flag() {
            Ok(ChangeKind::NotAliveDisposed)
        } else if !status_info.disposed_flag() && status_info.unregistered_flag() && !status_info.filtered_flag() {
            Ok(ChangeKind::NotAliveUnregistered)
        } else if !status_info.disposed_flag() && !status_info.unregistered_flag() && status_info.filtered_flag() {
                Ok(ChangeKind::AliveFiltered)
        } else if !status_info.disposed_flag() && !status_info.unregistered_flag() && !status_info.filtered_flag() {
                Ok(ChangeKind::Alive)
        } else {
            Err("Combination should not occur")
        }
    }
}

#[derive(PartialEq)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

pub type InstanceHandle = [u8; 16];

#[derive(PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

pub type VendorId = [u8; 2];

