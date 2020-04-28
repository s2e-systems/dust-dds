use std::convert::TryInto;
use num_derive::FromPrimitive;

// use crate::messages::{InlineQosParameter};
use serde::{Serialize, Serializer};
use serde::ser::{SerializeMap};
use serde_derive::{Deserialize, Serialize};
use std::{i32, u32};
use std::slice::Iter;
use std::collections::BTreeMap;
use std::ops::Index;
use crate::serdes::{RtpsSerialize, RtpsDeserialize, RtpsDeserializeWithEndianess, EndianessFlag, RtpsSerdesResult, RtpsSerdesError, PrimitiveSerdes, SizeCheckers};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct EntityKey([u8;3]);

impl<W> RtpsSerialize<W> for EntityKey
where 
    W: std::io::Write
{
    fn serialize(&self, writer: &mut W, _endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        writer.write(&self.0)?;

        Ok(())
    }
}

impl RtpsDeserialize for EntityKey{
    type Output = EntityKey;

    fn deserialize(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        SizeCheckers::check_size_equal(bytes, 3)?;

        Ok(EntityKey(bytes[0..3].try_into()?))
    }
}

#[derive(FromPrimitive, Debug, Hash, PartialEq, Eq, Clone, Copy)]
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

impl<W> RtpsSerialize<W> for EntityKind
where 
    W: std::io::Write
{
    fn serialize(&self, writer: &mut W, _endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        let entity_kind_u8 = *self as u8;
        writer.write(&[entity_kind_u8])?;

        Ok(())
    }
}

impl RtpsDeserialize for EntityKind{
    type Output = EntityKind;

    fn deserialize(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        SizeCheckers::check_size_equal(bytes, 1 /*expected_size*/)?;
        Ok(num::FromPrimitive::from_u8(bytes[0]).ok_or(RtpsSerdesError::InvalidEnumRepresentation)?)
    }
}

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
}

impl<W> RtpsSerialize<W> for EntityId
where 
    W: std::io::Write
{
    fn serialize(&self, writer: &mut W, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        self.entity_key.serialize(writer, endianness)?;
        self.entity_kind.serialize(writer, endianness)
    }
}

impl RtpsDeserialize for EntityId{
    type Output = EntityId;

    fn deserialize(bytes: &[u8]) -> RtpsSerdesResult<Self::Output> {
        SizeCheckers::check_size_equal(bytes, 4 /*expected_size*/)?;
        let entity_key = EntityKey::deserialize(&bytes[0..3])?;
        let entity_kind = EntityKind::deserialize(&[bytes[3]])?;

        Ok(EntityId::new(entity_key, entity_kind))
    }
}



pub const VENDOR_ID: VendorId = [99,99];


pub const ENTITYID_UNKNOWN: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x00]),
    entity_kind: EntityKind::UserDefinedUnknown,
};

pub const ENTITYID_PARTICIPANT: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x01]),
    entity_kind: EntityKind::BuiltInParticipant,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_ANNOUNCER: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x02]),
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_TOPICS_DETECTOR: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x02]),
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x03]),
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_PUBLICATIONS_DETECTOR: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x03]),
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_ANNOUNCER: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x04]),
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SEDP_BUILTIN_SUBSCRIPTIONS_DETECTOR: EntityId = EntityId {
    entity_key: EntityKey([0, 0, 0x04]),
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_ANNOUNCER: EntityId = EntityId {
    entity_key: EntityKey([0, 0x01, 0x00]),
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_SPDP_BUILTIN_PARTICIPANT_DETECTOR: EntityId = EntityId {
    entity_key: EntityKey([0, 0x01, 0x00]),
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_WRITER: EntityId = EntityId {
    entity_key: EntityKey([0, 0x02, 0x00]),
    entity_kind: EntityKind::BuiltInWriterWithKey,
};

pub const ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER: EntityId = EntityId {
    entity_key: EntityKey([0, 0x02, 0x00]),
    entity_kind: EntityKind::BuiltInReaderWithKey,
};

pub enum TopicKind {
    NoKey,
    WithKey,
}

pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
pub enum ChangeKind {
    Alive,
    AliveFiltered,
    NotAliveDisposed,
    NotAliveUnregistered,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Time {
    pub seconds: u32,
    pub fraction: u32,
}

impl Time {
    fn new (seconds: u32, fraction: u32) -> Self {
        Time {
            seconds,
            fraction,
        }
    }
}
 
impl<W> RtpsSerialize<W> for Time 
    where W: std::io::Write
{
    fn serialize(&self, writer: &mut W, endianness: EndianessFlag) -> RtpsSerdesResult<()>{
        let seconds_bytes = PrimitiveSerdes::serialize_u32(self.seconds, endianness);
        let fraction_bytes = PrimitiveSerdes::serialize_u32(self.fraction, endianness);

        writer.write(&seconds_bytes).unwrap();
        writer.write(&fraction_bytes).unwrap();

        Ok(())
    }
}

impl RtpsDeserializeWithEndianess for Time {
    type Output = Time;

    fn deserialize_with_endianness(bytes: &[u8], endianness: EndianessFlag) -> RtpsSerdesResult<Self::Output> {
        SizeCheckers::check_size_equal(bytes, 8)?;

        let seconds = PrimitiveSerdes::deserialize_u32(bytes[0..4].try_into()?, endianness);
        let fraction = PrimitiveSerdes::deserialize_u32(bytes[4..8].try_into()?, endianness);

        Ok(Time::new(seconds, fraction))
    }
}

const TIME_ZERO: Time = Time {
    seconds: 0,
    fraction: 0,
};
const TIME_INFINITE: Time = Time {
    seconds: u32::MAX,
    fraction: u32::MAX - 1,
};
const TIME_INVALID: Time = Time {
    seconds: u32::MAX,
    fraction: u32::MAX,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Duration {
    pub seconds: i32,
    pub fraction: u32,
}

pub const DURATION_ZERO: Duration = Duration {
    seconds: 0,
    fraction: 0,
};
pub const DURATION_INFINITE: Duration = Duration {
    seconds: i32::MAX,
    fraction: u32::MAX,
};

// pub type InlineQosParameterList = ParameterList<InlineQosParameter>;

pub trait Parameter {
    fn parameter_id(&self) -> u16;
}

#[derive(Hash, Clone, Debug, PartialEq, Eq)]
pub struct ParameterList<T: Parameter>(Vec<T>);

impl<T: Parameter> ParameterList<T> {
    pub fn new() -> Self {
        ParameterList(Vec::new())
    }

    pub fn new_from_vec(value: Vec<T>) -> Self {
        ParameterList(value)
    }

    pub fn iter(&self) -> Iter<'_,T>{
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, value: T) {
        self.0.push(value);
    }
}

impl<T: Parameter> Index<usize> for ParameterList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0.index(index)
    }
}

impl<T: Parameter + Serialize> Serialize for ParameterList<T> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        let mut map = serializer.serialize_map(None)?;

        for item in self.0.iter() {
            map.serialize_entry(&item.parameter_id(),item)?;
        }

        map.end()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone, Copy, Hash, Eq)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

#[derive(Serialize, Deserialize, PartialEq, Hash, Eq, Debug, Copy, Clone)]
pub struct Locator {
    pub kind: i32,
    pub port: u32,
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

#[derive(PartialEq, Deserialize, Debug, Eq, Hash)]
pub struct BuiltInEndPointSet {
    value: u32,
}

pub enum BuiltInEndPoints {
    ParticipantAnnouncer = 0,
    ParticipantDetector = 1,
    PublicationsAnnouncer = 2,
    PublicationsDetector = 3,
    SubscriptionsAnnouncer = 4,
    SubscriptionsDetector = 5,

    /*
    The following have been deprecated in version 2.4 of the
    specification. These bits should not be used by versions of the
    protocol equal to or newer than the deprecated version unless
    they are used with the same meaning as in versions prior to the
    deprecated version.
    @position(6) DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_ANNOUNCER,
    @position(7) DISC_BUILTIN_ENDPOINT_PARTICIPANT_PROXY_DETECTOR,
    @position(8) DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_ANNOUNCER,
    @position(9) DISC_BUILTIN_ENDPOINT_PARTICIPANT_STATE_DETECTOR,
    */
    ParticipantMessageDataWriter = 10,
    ParticipantMessageDataReader = 11,

    /*
    Bits 12-15 have been reserved by the DDS-Xtypes 1.2 Specification
    and future revisions thereof.
    Bits 16-27 have been reserved by the DDS-Security 1.1 Specification
    and future revisions thereof.
    */
    TopicsAnnouncer = 28,
    TopicsDetector = 29,
}

impl BuiltInEndPointSet {
    pub fn new(value: u32) -> Self {
        BuiltInEndPointSet { value }
    }

    pub fn has(&self, endpoint: BuiltInEndPoints) -> bool {
        let bit_position = endpoint as u8;
        let bitmask = 1 << bit_position;
        (self.value & bitmask) >> bit_position == 1
    }
}

pub type InstanceHandle = [u8; 16];
pub type VendorId = [u8; 2];
pub type LocatorList = Vec<Locator>;
pub type GuidPrefix = [u8; 12];
pub type Count = i32;
pub type SequenceNumber = i64;
pub type SequenceNumberSet = BTreeMap<SequenceNumber, bool>;
pub type FragmentNumber = u32;
pub type FragmentNumberSet = Vec<(FragmentNumber, bool)>;
pub type KeyHash = [u8; 16];
pub type StatusInfo = [u8; 4];

#[cfg(test)]
mod tests {
    use super::*;

    ///////////////////////// Entity Key Tests ////////////////////////
    #[test]
    fn test_entity_key_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_entity_key = EntityKey([5,20,250]);

        
        const TEST_ENTITY_KEY_BIG_ENDIAN : [u8;3] = [5,20,250];
        test_entity_key.serialize(&mut vec, EndianessFlag::BigEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KEY_BIG_ENDIAN);
        assert_eq!(EntityKey::deserialize(&vec).unwrap(), test_entity_key);
    }

    #[test]
    fn test_entity_key_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_entity_key = EntityKey([5,20,250]);

        
        const TEST_ENTITY_KEY_BIG_ENDIAN : [u8;3] = [5,20,250];
        test_entity_key.serialize(&mut vec, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KEY_BIG_ENDIAN);
        assert_eq!(EntityKey::deserialize(&vec).unwrap(), test_entity_key);
    }

    #[test]
    fn test_invalid_entity_key_deserialization() {
        let too_big_vec = vec![1,2,3,4];

        let expected_error = EntityKey::deserialize(&too_big_vec);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let too_small_vec = vec![1,2,3,4];

        let expected_error = EntityKey::deserialize(&too_small_vec);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };
    }


    ///////////////////////// Entity Kind Tests ////////////////////////

    #[test]
    fn test_entity_kind_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_entity_kind = EntityKind::BuiltInWriterWithKey;

        
        const TEST_ENTITY_KIND_BIG_ENDIAN : [u8;1] = [0xc2];
        test_entity_kind.serialize(&mut vec, EndianessFlag::BigEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KIND_BIG_ENDIAN);
        assert_eq!(EntityKind::deserialize(&vec).unwrap(), test_entity_kind);
    }

    #[test]
    fn test_entity_kind_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_entity_kind = EntityKind::BuiltInWriterWithKey;

        
        const TEST_ENTITY_KIND_LITTLE_ENDIAN : [u8;1] = [0xc2];
        test_entity_kind.serialize(&mut vec, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_KIND_LITTLE_ENDIAN);
        assert_eq!(EntityKind::deserialize(&vec).unwrap(), test_entity_kind);
    }

    #[test]
    fn test_invalid_entity_kind_deserialization() {
        let too_big_vec = vec![1,2,3,4];

        let expected_error = EntityKind::deserialize(&too_big_vec);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let wrong_vec = vec![0xf3];

        let expected_error = EntityKind::deserialize(&wrong_vec);
        match expected_error {
            Err(RtpsSerdesError::InvalidEnumRepresentation) => assert!(true),
            _ => assert!(false),
        };
    }

    ///////////////////////// Entity Id Tests ////////////////////////
    #[test]
    fn test_entity_id_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

        const TEST_ENTITY_ID_BIG_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
        test_entity_id.serialize(&mut vec, EndianessFlag::BigEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_ID_BIG_ENDIAN);
        assert_eq!(EntityId::deserialize(&vec).unwrap(), test_entity_id);
    }

    #[test]
    fn test_entity_id_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_entity_id = ENTITYID_BUILTIN_PARTICIPANT_MESSAGE_READER;

        const TEST_ENTITY_ID_LITTLE_ENDIAN : [u8;4] = [0, 0x02, 0x00, 0xc4];
        test_entity_id.serialize(&mut vec, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(vec, TEST_ENTITY_ID_LITTLE_ENDIAN);
        assert_eq!(EntityId::deserialize(&vec).unwrap(), test_entity_id);
    }

    #[test]
    fn test_invalid_entity_id_deserialization() {
        let too_big_vec = vec![1,2,3,4,5,5];

        let expected_error = EntityId::deserialize(&too_big_vec);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

        let wrong_vec = vec![1,2,3,0xf3];

        let expected_error = EntityId::deserialize(&wrong_vec);
        match expected_error {
            Err(RtpsSerdesError::InvalidEnumRepresentation) => assert!(true),
            _ => assert!(false),
        };
    }


    ///////////////////////// Time Tests ////////////////////////
    #[test]
    fn test_time_serialization_deserialization_big_endian() {
        let mut vec = Vec::new();
        let test_time = Time::new(1234567, 98765432);

        
        const TEST_TIME_BIG_ENDIAN : [u8;8] = [0x00, 0x12, 0xD6, 0x87, 0x05, 0xE3, 0x0A, 0x78];
        test_time.serialize(&mut vec, EndianessFlag::BigEndian).unwrap();
        assert_eq!(vec, TEST_TIME_BIG_ENDIAN);
        assert_eq!(Time::deserialize_with_endianness(&vec, EndianessFlag::BigEndian).unwrap(), test_time);
    }

    #[test]
    fn test_time_serialization_deserialization_little_endian() {
        let mut vec = Vec::new();
        let test_time = Time::new(1234567, 98765432);
        
        const TEST_TIME_LITTLE_ENDIAN : [u8;8] = [0x87, 0xD6, 0x12, 0x00, 0x78, 0x0A, 0xE3, 0x05];
        test_time.serialize(&mut vec, EndianessFlag::LittleEndian).unwrap();
        assert_eq!(vec, TEST_TIME_LITTLE_ENDIAN);
        assert_eq!(Time::deserialize_with_endianness(&vec, EndianessFlag::LittleEndian).unwrap(), test_time);
    }

    #[test]
    fn test_invalid_time_deserialization() {
        let wrong_vec = vec![1,2,3,4];

        let expected_error = Time::deserialize_with_endianness(&wrong_vec, EndianessFlag::LittleEndian);
        match expected_error {
            Err(RtpsSerdesError::WrongSize) => assert!(true),
            _ => assert!(false),
        };

    }

    #[test]
    fn test_builtin_endpoint_set_participant_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(1).has(BuiltInEndPoints::ParticipantAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::ParticipantAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::ParticipantAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_participant_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(2).has(BuiltInEndPoints::ParticipantDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::ParticipantDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::ParticipantDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::PublicationsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(4).has(BuiltInEndPoints::PublicationsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::PublicationsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::PublicationsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_publications_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::PublicationsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(8).has(BuiltInEndPoints::PublicationsDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::PublicationsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(15).has(BuiltInEndPoints::PublicationsDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(16).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(32).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(31).has(BuiltInEndPoints::SubscriptionsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_set_subscriptions_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::SubscriptionsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(32).has(BuiltInEndPoints::SubscriptionsDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(31).has(BuiltInEndPoints::SubscriptionsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(63).has(BuiltInEndPoints::SubscriptionsDetector),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_writer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(1024).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(1023).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(2047).has(BuiltInEndPoints::ParticipantMessageDataWriter),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_participant_message_data_reader() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::ParticipantMessageDataReader),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(2048).has(BuiltInEndPoints::ParticipantMessageDataReader),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(2047).has(BuiltInEndPoints::ParticipantMessageDataReader),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(4095).has(BuiltInEndPoints::ParticipantMessageDataReader),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_announcer() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::TopicsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(268435456).has(BuiltInEndPoints::TopicsAnnouncer),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(268435455).has(BuiltInEndPoints::TopicsAnnouncer),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(536870911).has(BuiltInEndPoints::TopicsAnnouncer),
            true
        );
    }

    #[test]
    fn test_builtin_endpoint_topics_detector() {
        assert_eq!(
            BuiltInEndPointSet::new(0).has(BuiltInEndPoints::TopicsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(536870912).has(BuiltInEndPoints::TopicsDetector),
            true
        );
        assert_eq!(
            BuiltInEndPointSet::new(536870911).has(BuiltInEndPoints::TopicsDetector),
            false
        );
        assert_eq!(
            BuiltInEndPointSet::new(1073741823).has(BuiltInEndPoints::TopicsDetector),
            true
        );
    }
}
