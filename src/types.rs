
use serde_derive::{Deserialize, Serialize};
use std::{u32, i32};
use crate::parser::InlineQosParameter;

#[derive(Serialize, Hash, Deserialize, Eq, PartialEq, Default, Debug)]
pub struct EntityId {
    entity_key: [u8;3],
    entity_kind: EntityKind,
}

impl EntityId{
    pub fn new(entity_key: [u8;3], entity_kind: u8) -> EntityId {
        EntityId {
            entity_key,
            entity_kind,
        }
    }
}

pub const ENTITY_KIND_BUILT_IN_UNKNOWN: EntityKind = 0xc0;
pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: EntityKind = 0xc1;
pub const ENTITY_KIND_WRITER_WITH_KEY: EntityKind = 0x02;
pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: EntityKind = 0xc2;
pub const ENTITY_KIND_WRITER_NO_KEY: EntityKind = 0x03;
pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: EntityKind = 0xc3;
pub const ENTITY_KIND_READER_NO_KEY: EntityKind = 0x04;
pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: EntityKind = 0xc4;
pub const ENTITY_KIND_READER_WITH_KEY: EntityKind = 0x07;
pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: EntityKind = 0xc7;
pub const ENTITY_KIND_WRITER_GROUP: EntityKind = 0x08;
pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: EntityKind = 0xc8;
pub const ENTITY_KIND_READER_GROUP: EntityKind = 0x09;
pub const ENTITY_KIND_BUILT_IN_READER_GROUP: EntityKind = 0xc9;

pub const ENTITYID_UNKNOWN: EntityId = EntityId{
    entity_key: [0,0,0x00], 
    entity_kind: 0};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId{
    entity_key: [0,0,0x01],
    entity_kind: ENTITY_KIND_BUILT_IN_PARTICIPANT};

pub const ENTITYID_SEDP_BUILT_IN_TOPIC_WRITER: EntityId = EntityId{
    entity_key: [0,0,0x02],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY};

pub const ENTITYID_SEDP_BUILT_IN_TOPIC_READER: EntityId = EntityId{
    entity_key: [0,0,0x02],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY};

pub const ENTITYID_SEDP_BUILT_IN_PUBLICATIONS_WRITER: EntityId = EntityId{
    entity_key: [0,0,0x03],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY};

pub const ENTITYID_SEDP_BUILT_IN_PUBLICATIONS_READER: EntityId = EntityId{
    entity_key: [0,0,0x03],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY};

pub const ENTITYID_SEDP_BUILT_IN_SUBSCRIPTIONS_WRITER: EntityId = EntityId{
    entity_key: [0,0,0x04],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY};

pub const ENTITYID_SEDP_BUILT_IN_SUBSCRIPTIONS_READER: EntityId = EntityId{
    entity_key: [0,0,0x04],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY};

pub const ENTITYID_SPDP_BUILT_IN_PARTICIPANT_WRITER: EntityId = EntityId{
    entity_key: [0,0x01,0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY};

pub const ENTITYID_SPDP_BUILT_IN_PARTICIPANT_READER: EntityId = EntityId{
    entity_key: [0,0x01,0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY};

pub const ENTITYID_BUILT_IN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId{
    entity_key: [0,0x02,0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY};

pub const ENTITYID_BUILT_IN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId{
    entity_key: [0,0x02,0x00],
    entity_kind: ENTITY_KIND_BUILT_IN_READER_WITH_KEY};

pub enum TopicKind {
    NoKey,
    WithKey,
}

pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

const TIME_ZERO: Time = Time{seconds: 0, fraction: 0 };
const TIME_INFINITE: Time = Time{seconds: u32::MAX, fraction: u32::MAX-1 };
const TIME_INVALID: Time = Time{seconds: u32::MAX, fraction: u32::MAX };

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

const DURATION_ZERO: Duration = Duration{seconds: 0, fraction: 0 };
const DURATION_INFINITE: Duration = Duration{seconds: i32::MAX, fraction: u32::MAX };

pub type InlineQosParameterList = Vec<InlineQosParameter>;

pub type ParameterList = Vec<Parameter>;

#[derive(Hash, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Parameter {
    pub parameter_id: u16,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Locator {
    pub kind: i32,
    pub port: u32,
    pub address: [u8;16],
}

impl Locator {
    pub fn new(kind: i32, port: u32, address: [u8;16]) -> Locator {
        Locator {
            kind,
            port,
            address,
        }
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Default, Debug)]
pub struct GUID {
    prefix: GuidPrefix,
    entity_id: EntityId,
}

impl GUID {
    pub fn new(prefix: GuidPrefix, entity_id: EntityId) -> GUID {
        GUID {
            prefix,
            entity_id,
        }
    }

    fn prefix(&self) -> &GuidPrefix {
        &self.prefix
    }

    fn entity_id(&self) -> &EntityId {
        &self.entity_id
    }

    fn new_participant_guid(prefix: GuidPrefix) -> GUID {
        GUID {
            prefix,
            entity_id: ENTITYID_PARTICIPANT,
        }
    }

    fn new_spdp_builtin_participant_reader_guid(prefix: GuidPrefix) -> GUID {
        GUID {
            prefix,
            entity_id: ENTITYID_SPDP_BUILT_IN_PARTICIPANT_READER,
        }
    }
}

pub type EntityKind = u8;
pub type InstanceHandle = [u8;16];
pub type VendorId = [u8;2];
pub type LocatorList = Vec<Locator>;
pub type GuidPrefix = [u8;12];
pub type Count = i32;
pub type SequenceNumber = i64;
pub type SequenceNumberSet = Vec<(SequenceNumber, bool)>;
pub type FragmentNumber = u32;
pub type FragmentNumberSet = Vec<(FragmentNumber, bool)>;
pub type KeyHash = [u8;16];
pub type StatusInfo = [u8;4];