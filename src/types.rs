
use serde_derive::{Deserialize, Serialize};

// pub type EntityId = [u8;4];

#[derive(Deserialize, Eq, PartialEq, Default, Debug)]
pub struct EntityId {
    entity_key: [u8;3],
    entity_kind: u8,
}

impl EntityId{
    pub fn new(entity_key: &[u8;3], entity_kind: &u8) -> EntityId {
        EntityId {
            entity_key: *entity_key,
            entity_kind: *entity_kind,
        }
    }
}

pub type TopicKind = u32;
pub type ReliabilityLevel = u32;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct TimeT {
    pub seconds: u32,
    pub fraction: u32,
}

pub type ParameterList = Vec<Parameter>;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Parameter {
    pub parameter_id: u16,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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

#[derive(PartialEq, Eq, Default)]
pub struct GUID {
    pub prefix: GuidPrefix,
    pub entity_id: EntityId,
}

pub type InstanceHandle = [u8;16];
pub type VendorId = [u8;2];
pub type LocatorList = Vec<Locator>;
pub type GuidPrefix = [u8;12];
pub type Count = i32;
pub type SequenceNumber = i64;
pub type SequenceNumberSet = Vec<(SequenceNumber, bool)>;
pub type FragmentNumber = u32;
pub type FragmentNumberSet = Vec<(FragmentNumber, bool)>;
