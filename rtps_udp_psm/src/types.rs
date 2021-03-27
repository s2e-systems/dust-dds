// impl EntityId {
//     pub const ENTITY_KIND_USER_DEFINED_UNKNOWN: u8 = 0x00;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_WITH_KEY: u8 = 0x02;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_NO_KEY: u8 = 0x03;
//     pub const ENTITY_KIND_USER_DEFINED_READER_WITH_KEY: u8 = 0x04;
//     pub const ENTITY_KIND_USER_DEFINED_READER_NO_KEY: u8 = 0x07;
//     pub const ENTITY_KIND_USER_DEFINED_WRITER_GROUP: u8 = 0x08;
//     pub const ENTITY_KIND_USER_DEFINED_READER_GROUP: u8 = 0x09;
//     pub const ENTITY_KIND_BUILT_IN_UNKNOWN: u8 = 0xc0;
//     pub const ENTITY_KIND_BUILT_IN_PARTICIPANT: u8 = 0xc1;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_WITH_KEY: u8 = 0xc2;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_NO_KEY: u8 = 0xc3;
//     pub const ENTITY_KIND_BUILT_IN_READER_WITH_KEY: u8 = 0xc4;
//     pub const ENTITY_KIND_BUILT_IN_READER_NO_KEY: u8 = 0xc7;
//     pub const ENTITY_KIND_BUILT_IN_WRITER_GROUP: u8 = 0xc8;
//     pub const ENTITY_KIND_BUILT_IN_READER_GROUP: u8 = 0xc9;

//     pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
//         entity_key: [0, 0, 0x01],
//         entity_kind: 0xc1,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x02],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x02],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x03],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x03],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0, 0x04],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0, 0x04],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
//         entity_key: [0, 0x01, 0x00],
//         entity_kind: 0xc2,
//     };

//     pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
//         entity_key: [0, 0x01, 0x00],
//         entity_kind: 0xc7,
//     };

//     pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
//         entity_key: [0, 0x02, 0x00],
//         entity_kind: 0xc2,
//     };
//     pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
//         entity_key: [0, 0x02, 0x00],
//         entity_kind: 0xc7,
//     };
// }

// impl rust_rtps_pim::types::EntityId for EntityId {
//     const ENTITYID_UNKNOWN: Self = Self {
//         entity_key: [0; 3],
//         entity_kind: 0,
//     };
// }

pub type GuidPrefix = [u8; 12];

#[derive(Clone, Copy)]
pub struct EntityId {
    pub entity_key: [u8; 3],
    pub entity_kind: u8,
}

impl Into<[u8;4]> for EntityId {
    fn into(self) -> [u8;4] {
        [self.entity_key[0], self.entity_key[1], self.entity_key[2], self.entity_kind]
    }
}

impl From<[u8;4]> for EntityId {
    fn from(value: [u8;4]) -> Self {
        Self {
            entity_key: [value[0], value[1], value[2]],
            entity_kind: value[3]
        }
    }
}

#[derive(Clone, Copy)]
pub struct Guid {
    pub prefix: GuidPrefix,
    pub entity_id: EntityId,
}

impl Into<[u8;16]> for Guid {
    fn into(self) -> [u8;16] {
        todo!()
    }
}

impl From<[u8;16]> for Guid {
    fn from(_: [u8;16]) -> Self {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SequenceNumber {
    pub high: i32,
    pub low: u32,
}

impl Into<i64> for SequenceNumber {
    fn into(self) -> i64 {
        ((self.high as i64) << 32) + self.low as i64
    }
}

impl From<i64> for SequenceNumber {
    fn from(value: i64) -> Self {
        Self {
            high: (value >> 32) as i32,
            low: value as u32,
        }
    }
}

pub struct Locator {
    pub kind: <Self as rust_rtps_pim::structure::types::Locator>::Kind,
    pub port: <Self as rust_rtps_pim::structure::types::Locator>::Port,
    pub address: <Self as rust_rtps_pim::structure::types::Locator>::Address,
}
impl rust_rtps_pim::structure::types::Locator for Locator {
    type Kind = i32;
    type Port = u32;
    type Address = [u8; 16];

    const LOCATOR_KIND_INVALID: Self::Kind = -1;
    const LOCATOR_KIND_RESERVED: Self::Kind = 0;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv4: Self::Kind = 1;
    #[allow(non_upper_case_globals)]
    const LOCATOR_KIND_UDPv6: Self::Kind = 2;
    const LOCATOR_ADDRESS_INVALID: Self::Address = [0; 16];
    const LOCATOR_PORT_INVALID: Self::Port = 0;
}

pub type ReliabilityKind = i32;

pub type InstanceHandle = i32;

#[derive(Clone, Copy)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

#[derive(Clone, Copy)]
pub enum TopicKind {
    NoKey,
    WithKey,
}

#[derive(Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

pub type VendorId = [u8;2];

pub type ProtocolId = [u8;4];
pub type SubmessageFlag = bool;

#[derive(Clone, Copy)]
pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

pub type Count = i32;
pub type ParameterId = i16;
pub type FragmentNumber = u32;
pub type GroupDigest = [u8;4];

#[derive(Clone, Copy)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

#[derive(Clone, Copy)]
pub enum ChangeForReaderStatusKind {
    Unsent,
    Unacknowledged,
    Requested,
    Acknowledged,
    Underway,
}

#[derive(Clone, Copy)]
pub enum ChangeFromWriterStatusKind {
    Lost,
    Missing,
    Received,
    Unknown,
}

pub struct Parameter {
    pub parameter_id: ParameterId,
    pub length: i16,
    pub value: Vec<u8>,
}

impl rust_rtps_pim::messages::submessage_elements::Parameter for Parameter {
    type ParameterId = ParameterId;

    fn parameter_id(&self) -> Self::ParameterId {
        self.parameter_id
    }

    fn length(&self) -> i16 {
        self.length
    }

    fn value(&self) -> &[u8] {
        &self.value
    }
}