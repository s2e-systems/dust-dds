use rust_dds_types::InstanceHandle;
use serde::{Deserialize, Serialize};
use std::convert::{TryFrom, TryInto};
///
/// This files shall only contain the types as listed in the DDSI-RTPS Version 2.3
/// Table 8.2 - Types of the attributes that appear in the RTPS Entities and Classes
///  

pub mod constants {
    use super::{
        EntityId, EntityKey, EntityKind, GuidPrefix, Locator, ProtocolVersion, VendorId, GUID,
    };

    pub const VENDOR_ID: VendorId = [99, 99];

    pub const PROTOCOL_VERSION_2_1: ProtocolVersion = ProtocolVersion { major: 2, minor: 1 };
    pub const PROTOCOL_VERSION_2_2: ProtocolVersion = ProtocolVersion { major: 2, minor: 2 };
    pub const PROTOCOL_VERSION_2_4: ProtocolVersion = ProtocolVersion { major: 2, minor: 4 };

    pub const LOCATOR_KIND_INVALID: i32 = -1;
    pub const LOCATOR_KIND_RESERVED: i32 = 0;
    #[allow(non_upper_case_globals)]
    pub const LOCATOR_KIND_UDPv4: i32 = 1;
    #[allow(non_upper_case_globals)]
    pub const LOCATOR_KIND_UDPv6: i32 = 2;
    pub const LOCATOR_PORT_INVALID: u32 = 0;
    pub const LOCATOR_ADDRESS_INVALID: [u8; 16] = [0; 16];
    pub const LOCATOR_INVALID: Locator = Locator::new(
        LOCATOR_KIND_INVALID,
        LOCATOR_PORT_INVALID,
        LOCATOR_ADDRESS_INVALID,
    );

    pub const GUID_PREFIX_UNKNOWN: GuidPrefix = [0; 12];

    pub const ENTITY_KIND_USER_DEFINED_UNKNOWN: EntityKind = 0x00;
    pub const ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY: EntityKind = 0x02;
    pub const ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY: EntityKind = 0x03;
    pub const ENTITY_KIND_USER_DEFINED_READER_WITH_KEY: EntityKind = 0x04;
    pub const ENTITY_KIND_USER_DEFINED_READER_NO_KEY: EntityKind = 0x07;
    pub const ENTITY_KIND_USER_DEFINED_WRITER_GROUP: EntityKind = 0x08;
    pub const ENTITY_KIND_USER_DEFINED_READER_GROUP: EntityKind = 0x09;
    pub const ENTITY_KIND_BUILT_IN_UNKNOWN: EntityKind = 0xc0;
    pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: EntityKind = 0xc1;
    pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: EntityKind = 0xc2;
    pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: EntityKind = 0xc3;
    pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: EntityKind = 0xc4;
    pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: EntityKind = 0xc7;
    pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: EntityKind = 0xc8;
    pub const ENTITY_KIND_BUILT_IN_READER_GROUP: EntityKind = 0xc9;

    pub const ENTITYID_UNKNOWN: EntityId = EntityId {
        entity_key: [0, 0, 0x00],
        entity_kind: ENTITY_KIND_USER_DEFINED_UNKNOWN,
    };

    pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
        entity_key: [0, 0, 0x01],
        entity_kind: ENTITY_KIND_BUILT_IN_PARTICIPANT,
    };

    pub const ENTITYKEY_SEDP_BUILTIN_TOPICS: EntityKey = [0, 0, 0x02];
    pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
        entity_key: ENTITYKEY_SEDP_BUILTIN_TOPICS,
        entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
    };
    pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
        entity_key: ENTITYKEY_SEDP_BUILTIN_TOPICS,
        entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
    };

    pub const ENTITYKEY_SEDP_BUILTIN_PUBLICATIONS: EntityKey = [0, 0, 0x03];
    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: ENTITYKEY_SEDP_BUILTIN_PUBLICATIONS,
        entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
    };
    pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
        entity_key: ENTITYKEY_SEDP_BUILTIN_PUBLICATIONS,
        entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
    };

    pub const ENTITYKEY_SEDP_BUILTIN_SUBSCRIPTIONS: EntityKey = [0, 0, 0x04];
    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
        entity_key: ENTITYKEY_SEDP_BUILTIN_SUBSCRIPTIONS,
        entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
    };
    pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
        entity_key: ENTITYKEY_SEDP_BUILTIN_SUBSCRIPTIONS,
        entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
    };

    pub const ENTITYKEY_SPDP_BUILTIN_PARTICIPANT: EntityKey = [0, 0x01, 0x00];
    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
        entity_key: ENTITYKEY_SPDP_BUILTIN_PARTICIPANT,
        entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
    };

    pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
        entity_key: ENTITYKEY_SPDP_BUILTIN_PARTICIPANT,
        entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
    };

    pub const ENTITYKEY_BUILTIN_PARTICIPANT_MESSAGE: EntityKey = [0, 0x02, 0x00];
    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
        entity_key: ENTITYKEY_BUILTIN_PARTICIPANT_MESSAGE,
        entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY,
    };
    pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
        entity_key: ENTITYKEY_BUILTIN_PARTICIPANT_MESSAGE,
        entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY,
    };

    pub const GUID_UNKNOWN: GUID = GUID::new(GUID_PREFIX_UNKNOWN, ENTITYID_UNKNOWN);
}

#[derive(PartialEq, Copy, Clone)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
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

impl From<GUID> for InstanceHandle {
    fn from(guid: GUID) -> Self {
        let mut instance_handle = [0u8; 16];
        instance_handle[0..12].copy_from_slice(&guid.prefix);
        instance_handle[12..15].copy_from_slice(&guid.entity_id.entity_key);
        instance_handle[15] = guid.entity_id.entity_kind as u8;
        instance_handle
    }
}

impl TryFrom<InstanceHandle> for GUID {
    type Error = ();
    fn try_from(value: InstanceHandle) -> Result<Self, Self::Error> {
        let prefix = value[0..12].try_into().unwrap();
        let entity_key = value[12..15].try_into().unwrap();
        let entity_kind = value[15];
        let entity_id = EntityId::new(entity_key, entity_kind);
        Ok(GUID::new(prefix, entity_id))
    }
}

pub type GuidPrefix = [u8; 12];

pub type EntityKey = [u8; 3];

pub type EntityKind = u8;

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
    pub fn entity_key(&self) -> EntityKey {
        self.entity_key
    }
    pub fn entity_kind(&self) -> EntityKind {
        self.entity_kind
    }
}

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
        let address: [u8; 16] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, address[0], address[1], address[2], address[3],
        ];
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

    pub fn address(&self) -> &[u8; 16] {
        &self.address
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Hash, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

pub type VendorId = [u8; 2];



pub type SequenceNumber = i64;

pub const SEQUENCE_NUMBER_UNKNOWN: SequenceNumber = std::i64::MIN;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

pub type ParameterId = i16;

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    parameter_id: ParameterId,
    length: i16, // length is rounded up to multple of 4
    value: Vec<u8>,
}

impl Parameter {
    pub fn new(parameter_id: ParameterId, value: Vec<u8>) -> Self {
        Self {
            parameter_id,
            length: (value.len() + 3 & !3) as i16,
            value,
        }
    }

    pub fn parameter_id(&self) -> ParameterId {
        self.parameter_id
    }

    pub fn length(&self) -> i16 {
        self.length
    }

    pub fn value(&self) -> &Vec<u8> {
        &self.value
    }
}
#[derive(Debug, PartialEq, Clone)]
pub struct ParameterList {
    pub parameter: Vec<Parameter>,
}

impl ParameterList {
    pub const PID_SENTINEL: ParameterId = 0x0001;

    pub fn new() -> Self {
        Self {
            parameter: Vec::new(),
        }
    }
}

//// From RTPS
#[derive(Copy, Clone)]
pub enum TopicKind {
    NoKey,
    WithKey,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::InstanceHandle;

    #[test]
    fn guid_to_instance_handle() {
        let prefix = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0];
        let entity_id = EntityId::new(
            [200, 100, 5],
            constants::ENTITY_KIND_USER_DEFINED_READER_GROUP,
        );
        let guid = GUID::new(prefix, entity_id);

        let expected_instance_handle = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 0, 200, 100, 5, 9];

        assert_eq!(expected_instance_handle, InstanceHandle::from(guid));
    }
}
